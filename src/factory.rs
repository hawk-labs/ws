use communication::Sender;
use handler::Handler;


pub trait Factory {
    type Handler: Handler;

    fn connection_made(&mut self, _: Sender) -> Self::Handler;

    #[inline]
    fn on_shutdown(&mut self) {
        debug!("Factory received WebSocket shutdown request.");
    }

    #[inline]
    fn client_connected(&mut self, ws: Sender) -> Self::Handler {
        self.connection_made(ws)
    }

    #[inline]
    fn server_connected(&mut self, ws: Sender) -> Self::Handler {
        self.connection_made(ws)
    }

    #[inline]
    fn connection_lost(&mut self, _: Self::Handler) {}
}

impl<F, H> Factory for F
where
    H: Handler,
    F: FnMut(Sender) -> H,
{
    type Handler = H;

    fn connection_made(&mut self, out: Sender) -> H {
        self(out)
    }
}

mod test {
    #![allow(unused_imports, unused_variables, dead_code)]
    use super::*;
    use communication::{Command, Sender};
    use frame;
    use handler::Handler;
    use handshake::{Handshake, Request, Response};
    use message;
    use mio;
    use protocol::CloseCode;
    use result::Result;

    #[derive(Debug, Eq, PartialEq)]
    struct M;
    impl Handler for M {
        fn on_message(&mut self, _: message::Message) -> Result<()> {
            Ok(println!("test"))
        }

        fn on_frame(&mut self, f: frame::Frame) -> Result<Option<frame::Frame>> {
            Ok(None)
        }
    }

    #[test]
    fn impl_factory() {
        struct X;

        impl Factory for X {
            type Handler = M;
            fn connection_made(&mut self, _: Sender) -> M {
                M
            }
        }

        let (chn, _) = mio::channel::sync_channel(42);

        let mut x = X;
        let m = x.connection_made(Sender::new(mio::Token(0), chn, 0));
        assert_eq!(m, M);
    }

    #[test]
    fn closure_factory() {
        let (chn, _) = mio::channel::sync_channel(42);

        let mut factory = |_| |_| Ok(());

        factory.connection_made(Sender::new(mio::Token(0), chn, 0));
    }

    #[test]
    fn connection_lost() {
        struct X;

        impl Factory for X {
            type Handler = M;
            fn connection_made(&mut self, _: Sender) -> M {
                M
            }
            fn connection_lost(&mut self, handler: M) {
                assert_eq!(handler, M);
            }
        }

        let (chn, _) = mio::channel::sync_channel(42);

        let mut x = X;
        let m = x.connection_made(Sender::new(mio::Token(0), chn, 0));
        x.connection_lost(m);
    }
}

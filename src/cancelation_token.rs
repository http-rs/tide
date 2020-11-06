use {
	std::{
        future::Future,
		pin::Pin,
		sync::{Arc, Mutex},
		task::{Context, Poll, Waker},
	},
};

#[derive(Debug)]
pub struct CancelationToken {
	shared_state: Arc<Mutex<CancelationTokenState>>
}

#[derive(Debug)]
struct CancelationTokenState {
	canceled: bool,
	waker: Option<Waker>
}

impl CancelationToken {
	pub fn new() -> CancelationToken {
		CancelationToken {
			shared_state: Arc::new(Mutex::new(CancelationTokenState {
				canceled: false,
				waker: None
			}))
		}
	}

	pub fn complete(&self) {
		let mut shared_state = self.shared_state.lock().unwrap();

		shared_state.canceled = true;
		if let Some(waker) = shared_state.waker.take() {
			waker.wake()
		}
	}
}

impl Future for CancelationToken {
	type Output = ();

	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		let mut shared_state = self.shared_state.lock().unwrap();

		if shared_state.canceled {
            Poll::Ready(())
		} else {
            shared_state.waker = Some(cx.waker().clone());
            Poll::Pending
		}
	}
}

impl Clone for CancelationToken {
	fn clone(&self) -> Self {
		CancelationToken {
			shared_state: self.shared_state.clone()
		}
	}
}
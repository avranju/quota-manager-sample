use std::future::Future;
use std::{cell::RefCell, collections::HashMap, pin::Pin, rc::Rc, task::Context, task::Poll};

use futures::future::{self, Ready};
use ntex::Service;
use pin_project::pin_project;

use crate::{error::Error, hub::Hub};

#[derive(Clone)]
pub struct QuotaManager {
    state: Rc<RefCell<QuotaInner>>,
}

struct QuotaInner {
    count_quota: HashMap<String, u64>,
}

impl QuotaManager {
    pub fn new(count_quota: HashMap<String, u64>) -> Self {
        QuotaManager {
            state: Rc::new(RefCell::new(QuotaInner { count_quota })),
        }
    }

    pub fn enforce_message_quota(&self, hub: Hub) -> Ready<Result<(), Error>> {
        if let Some(max_count) = self.state.borrow().count_quota.get(&hub.id()) {
            if hub.message_count() < *max_count {
                future::ok(())
            } else {
                future::err(Error("Message quota exceeded".to_string()))
            }
        } else {
            future::ok(())
        }
    }
}

#[derive(Clone)]
pub struct QuotaService<S>
where
    S: Service,
{
    state: Rc<RefCell<QuotaServiceState<S>>>,
}

impl<S> QuotaService<S>
where
    S: Service,
{
    pub fn new(service: S, hub: Hub, quota_manager: QuotaManager) -> Self {
        QuotaService {
            state: Rc::new(RefCell::new(QuotaServiceState {
                service,
                hub,
                quota_manager,
                req: None,
            })),
        }
    }
}

pub struct QuotaServiceState<S>
where
    S: Service,
{
    service: S,
    req: Option<S::Request>,
    hub: Hub,
    quota_manager: QuotaManager,
}

impl<S> Service for QuotaService<S>
where
    S: Service<Request = String, Error = Error> + 'static,
{
    type Request = S::Request;
    type Response = S::Response;
    type Error = S::Error;
    type Future = QuotaServiceResponse<S, Ready<Result<(), Self::Error>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.state.borrow().service.poll_ready(cx)
    }

    fn call(&self, req: S::Request) -> Self::Future {
        self.state.borrow_mut().req = Some(req);
        let hub = self.state.borrow().hub.clone();
        let fut = self.state.borrow().quota_manager.enforce_message_quota(hub);

        QuotaServiceResponse::QuotaCheck(fut, self.state.clone())
    }
}

#[pin_project(project = QuotaServiceResponseProj)]
pub enum QuotaServiceResponse<S, F>
where
    S: Service,
{
    QuotaCheck(#[pin] F, Rc<RefCell<QuotaServiceState<S>>>),
    ServiceCall(#[pin] S::Future),
}

impl<S, F> Future for QuotaServiceResponse<S, F>
where
    S: Service<Error = Error>,
    F: Future<Output = Result<(), Error>>,
{
    type Output = <S::Future as Future>::Output;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.as_mut().project() {
            QuotaServiceResponseProj::QuotaCheck(fut, state) => match fut.poll(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(Err(err)) => Poll::Ready(Err(err)),
                Poll::Ready(Ok(_)) => {
                    // quota check succeeded; let's proceed with the call to
                    // the inner service
                    let req = state
                        .borrow_mut()
                        .req
                        .take()
                        .expect("Request must be populated");
                    let fut = state.borrow().service.call(req);
                    self.set(QuotaServiceResponse::ServiceCall(fut));
                    self.poll(cx)
                }
            },
            QuotaServiceResponseProj::ServiceCall(fut) => match fut.poll(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(res) => Poll::Ready(res),
            },
        }
    }
}

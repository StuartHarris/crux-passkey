use crux_core::capability::{CapabilityContext, Operation};
use crux_macros::Capability;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum PasskeyOperation {
    Register(String),
    Login(String),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum PasskeyOutput {
    Registered,
    LoggedIn,
}

impl Operation for PasskeyOperation {
    type Output = PasskeyOutput;
}

#[derive(Capability)]
pub struct Passkey<Ev> {
    context: CapabilityContext<PasskeyOperation, Ev>,
}

impl<Ev> Passkey<Ev>
where
    Ev: 'static,
{
    pub fn new(context: CapabilityContext<PasskeyOperation, Ev>) -> Self {
        Self { context }
    }

    pub fn register(&self, user_name: String, event: Ev)
    where
        Ev: Send,
    {
        let ctx = self.context.clone();
        self.context.spawn(async move {
            ctx.request_from_shell(PasskeyOperation::Register(user_name))
                .await;

            ctx.update_app(event);
        });
    }

    pub fn login(&self, user_name: String, event: Ev)
    where
        Ev: Send,
    {
        let ctx = self.context.clone();
        self.context.spawn(async move {
            ctx.request_from_shell(PasskeyOperation::Login(user_name))
                .await;

            ctx.update_app(event);
        });
    }
}

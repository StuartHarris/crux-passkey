use crux_core::capability::{CapabilityContext, Operation};
use crux_macros::Capability;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum PasskeyOperation {
    CreateCredential(Vec<u8>),
    RequestCredential(Vec<u8>),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum PasskeyOutput {
    RegisterCredential(Vec<u8>),
    Credential(Vec<u8>),
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

    pub fn create_credential<F, T>(&self, challenge: Vec<u8>, make_event: F)
    where
        F: Fn(T) -> Ev + Clone + Send + 'static,
        T: DeserializeOwned,
    {
        let ctx = self.context.clone();
        self.context.spawn(async move {
            let cred = ctx
                .request_from_shell(PasskeyOperation::CreateCredential(challenge))
                .await;

            if let PasskeyOutput::RegisterCredential(cred) = cred {
                let cred = serde_json::from_slice(&cred).expect("Failed to deserialize cred");
                let event = make_event(cred);
                ctx.update_app(event);
            }
        });
    }

    pub fn request_credential<F, T>(&self, challenge: Vec<u8>, make_event: F)
    where
        F: Fn(T) -> Ev + Clone + Send + 'static,
        T: DeserializeOwned,
    {
        let ctx = self.context.clone();
        self.context.spawn(async move {
            let cred = ctx
                .request_from_shell(PasskeyOperation::RequestCredential(challenge))
                .await;

            if let PasskeyOutput::Credential(cred) = cred {
                let cred = serde_json::from_slice(&cred).expect("Failed to deserialize cred");
                let event = make_event(cred);
                ctx.update_app(event);
            }
        });
    }
}

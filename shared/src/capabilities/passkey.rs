use crux_core::capability::{CapabilityContext, Operation};
use crux_macros::Capability;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Error {
    message: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum PasskeyOperation {
    CreateCredential(Vec<u8>),
    RequestCredential(Vec<u8>),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum PasskeyOutput {
    RegisterCredential(Vec<u8>),
    Credential(Vec<u8>),
    Error(String),
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
        F: FnOnce(Result<T>) -> Ev + Clone + Send + 'static,
        T: DeserializeOwned,
    {
        self.perform_operation(PasskeyOperation::CreateCredential(challenge), make_event);
    }

    pub fn request_credential<F, T>(&self, challenge: Vec<u8>, make_event: F)
    where
        F: FnOnce(Result<T>) -> Ev + Clone + Send + 'static,
        T: DeserializeOwned,
    {
        self.perform_operation(PasskeyOperation::RequestCredential(challenge), make_event);
    }

    fn perform_operation<F, T>(&self, op: PasskeyOperation, make_event: F)
    where
        F: FnOnce(Result<T>) -> Ev + Clone + Send + 'static,
        T: DeserializeOwned,
    {
        let ctx = self.context.clone();
        self.context.spawn(async move {
            let output = ctx.request_from_shell(op).await;

            let event = match output {
                PasskeyOutput::RegisterCredential(cred) | PasskeyOutput::Credential(cred) => {
                    let cred = serde_json::from_slice(&cred).expect("Failed to deserialize cred");
                    make_event(Ok(cred))
                }
                PasskeyOutput::Error(e) => make_event(Err(Error { message: e })),
            };

            ctx.update_app(event);
        });
    }
}

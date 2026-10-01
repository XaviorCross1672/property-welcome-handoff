use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use property_welcome_handoff::infrai_client::InfraiClient;
use property_welcome_handoff::tenant_welcome::{
    onboard, OnboardingError, OnboardingReceipt, OnboardingRequest,
};
use serde::Serialize;

#[tokio::main]
async fn main() {
    let client = InfraiClient::from_env().expect("INFRAI_API_KEY must be set");
    let app = Router::new()
        .route("/tenant-onboarding", post(handler))
        .with_state(client);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("bind service");
    println!("property onboarding listening on http://127.0.0.1:3000");
    axum::serve(listener, app).await.expect("serve requests");
}

async fn handler(
    State(client): State<InfraiClient>,
    Json(input): Json<OnboardingRequest>,
) -> Result<(StatusCode, Json<OnboardingReceipt>), ServiceError> {
    Ok((StatusCode::CREATED, Json(onboard(&client, input).await?)))
}

struct ServiceError(OnboardingError);

impl From<OnboardingError> for ServiceError {
    fn from(value: OnboardingError) -> Self {
        Self(value)
    }
}

#[derive(Serialize)]
struct Problem {
    error: String,
}

impl IntoResponse for ServiceError {
    fn into_response(self) -> Response {
        let status = match &self.0 {
            OnboardingError::ConsentRequired => StatusCode::FORBIDDEN,
            OnboardingError::NoDeliverableChannel => StatusCode::UNPROCESSABLE_ENTITY,
            OnboardingError::Infrai(error) => error.public_status(),
        };
        (
            status,
            Json(Problem {
                error: self.0.to_string(),
            }),
        )
            .into_response()
    }
}

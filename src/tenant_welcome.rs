use crate::infrai_client::{InfraiClient, InfraiError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SignupChannel {
    Email,
    Sms,
}

#[derive(Debug, Deserialize)]
pub struct OnboardingRequest {
    pub request_id: String,
    pub user_id: String,
    pub tenant_name: String,
    pub email: String,
    pub phone: String,
    pub signup_channel: SignupChannel,
    pub maintenance_requests: Vec<String>,
    pub tenant_documents: Vec<String>,
    pub inspection_reminders: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct OnboardingReceipt {
    pub user_id: String,
    pub channel: DeliveryChannel,
    pub message_id: String,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DeliveryChannel {
    Email,
    Sms,
}

#[derive(Debug, thiserror::Error)]
pub enum OnboardingError {
    #[error("onboarding consent is required")]
    ConsentRequired,
    #[error("both welcome destinations are suppressed")]
    NoDeliverableChannel,
    #[error(transparent)]
    Infrai(#[from] InfraiError),
}

pub fn choose_channel(
    preferred: SignupChannel,
    email_suppressed: bool,
    sms_suppressed: bool,
) -> Result<DeliveryChannel, OnboardingError> {
    match preferred {
        SignupChannel::Email if !email_suppressed => Ok(DeliveryChannel::Email),
        SignupChannel::Sms if !sms_suppressed => Ok(DeliveryChannel::Sms),
        SignupChannel::Email if !sms_suppressed => Ok(DeliveryChannel::Sms),
        SignupChannel::Sms if !email_suppressed => Ok(DeliveryChannel::Email),
        _ => Err(OnboardingError::NoDeliverableChannel),
    }
}

pub async fn onboard(
    client: &InfraiClient,
    input: OnboardingRequest,
) -> Result<OnboardingReceipt, OnboardingError> {
    if !client.onboarding_consent(&input.user_id).await? {
        return Err(OnboardingError::ConsentRequired);
    }
    let (email_suppressed, sms_suppressed) = tokio::try_join!(
        client.email_suppressed(&input.email),
        client.sms_suppressed(&input.phone),
    )?;
    let channel = choose_channel(input.signup_channel, email_suppressed, sms_suppressed)?;
    let message = format!(
        "Hello {}, your portal has {} maintenance request(s), {} tenant document(s), and {} inspection reminder(s).",
        input.tenant_name, input.maintenance_requests.len(), input.tenant_documents.len(), input.inspection_reminders.len()
    );
    let message_id = match channel {
        DeliveryChannel::Email => {
            client
                .send_email(&input.email, &message, &input.request_id)
                .await?
        }
        DeliveryChannel::Sms => {
            client
                .send_sms(&input.phone, &message, &input.request_id)
                .await?
        }
    };
    Ok(OnboardingReceipt {
        user_id: input.user_id,
        channel,
        message_id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suppressed_signup_email_hands_welcome_to_sms() {
        let selected = choose_channel(SignupChannel::Email, true, false).unwrap();
        assert_eq!(selected, DeliveryChannel::Sms);
    }
}

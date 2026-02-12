struct ForgotPassword;

impl ForgotPassword {
    pub fn new() -> Self {
        ForgotPassword
    }

    pub fn reset_password_with_email(&self, email: String) -> Result<(), String> {
        Ok(())
    }

    pub fn reset_password_with_phone(&self, phone_number: String) -> Result<(), String> {
        Ok(())
    }
}
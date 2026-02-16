struct ResetPassword;

impl ResetPassword {
    pub fn new() -> Self {
        ResetPassword
    }

    pub fn reset_passsword(&self, email: String) -> Result<(), String> {
        Ok(())
    }
}
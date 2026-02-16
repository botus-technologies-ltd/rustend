struct SignUp;

impl SignUp {
    type Email = String;
    type PhoneNumber = String;

    pub fn new() -> Self {
        SignUp
    }

    pub fn signup_with_email(&self, email: Email) -> Result<(), String> {
        Ok(())
    }

    pub fn signup_with_phone(&self, phone_number: PhoneNumber) -> Result<(), String> {
        Ok(())
    }

    pub fn signup_with_google(&self) -> Result<(), String> {
        Ok(())
    }

    pub fn signup_with_facebook(&self) -> Result<(), String> {
        Ok(())
    }

    pub fn signup_with_apple(&self) -> Result<(), String> {
        Ok(())
    }

    pub fn signup_with_twitter(&self) -> Result<(), String> {
        Ok(())
    }

    pub fn signup_with_github(&self) -> Result<(), String> {
        Ok(())
    }

    pub fn signup_with_linkedin(&self) -> Result<(), String> {
        Ok(())
    }   

    pub fn signup_with_instagram(&self) -> Result<(), String> {
        Ok(())
    }

    pub fn signup_with_twitch(&self) -> Result<(), String> {
        Ok(())
    }

    pub fn signup_with_discord(&self) -> Result<(), String> {
        Ok(())
    }

    pub fn signup_with_slack(&self) -> Result<(), String> {
        Ok(())
    }

    pub fn signup_with_microsoft(&self) -> Result<(), String> {
        Ok(())
    }
}
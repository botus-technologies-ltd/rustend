struct SighIn;

impl SighIn {
    pub fn new() -> Self {
        SighIn
    }

    pub fn sign_in_with_email(&self, email: String) -> Result<(), String> {
        Ok(())
    }

    pub fn sign_in_with_phone(&self, phone_number: String) -> Result<(), String> {
        Ok(())
    }

    pub fn sign_in_with_google(&self) -> Result<(), String> {
        Ok(())
    } 
}
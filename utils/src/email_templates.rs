//! Professional Email Templates Module
//!
//! Provides reusable email templates with consistent professional design.
//! All templates support customizable header text and can be used across the workspace.

use crate::email::Email;

/// Email template configuration
pub struct EmailTemplateConfig {
    pub app_name: String,
    pub app_logo_url: Option<String>,
    pub primary_color: String,
    pub secondary_color: String,
    pub frontend_url: String,
}

impl EmailTemplateConfig {
    pub fn new(app_name: impl Into<String>, frontend_url: impl Into<String>) -> Self {
        Self {
            app_name: app_name.into(),
            app_logo_url: None,
            primary_color: "#667eea".to_string(),
            secondary_color: "#764ba2".to_string(),
            frontend_url: frontend_url.into(),
        }
    }

    pub fn with_logo(mut self, logo_url: impl Into<String>) -> Self {
        self.app_logo_url = Some(logo_url.into());
        self
    }

    pub fn with_colors(mut self, primary: impl Into<String>, secondary: impl Into<String>) -> Self {
        self.primary_color = primary.into();
        self.secondary_color = secondary.into();
        self
    }
}

/// Build the professional HTML wrapper with consistent design
fn build_html_wrapper(config: &EmailTemplateConfig, header_text: &str, content: &str) -> String {
    let logo_html = config.app_logo_url.as_ref().map_or(String::new(), |logo| {
        format!(
            r#"<img src="{}" alt="{} Logo" style="height: 50px; margin-bottom: 10px;">"#,
            logo, config.app_name
        )
    });

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{} - {}</title>
</head>
<body style="font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif; line-height: 1.6; color: #333; max-width: 600px; margin: 0 auto; padding: 0; background-color: #f4f4f4;">
    <table width="100%" cellpadding="0" cellspacing="0" style="background-color: #f4f4f4; padding: 20px 0;">
        <tr>
            <td align="center">
                <table width="100%" cellpadding="0" cellspacing="0" style="max-width: 600px; background-color: #ffffff; border-radius: 8px; overflow: hidden; box-shadow: 0 2px 4px rgba(0,0,0,0.1);">
                    <!-- Header -->
                    <tr>
                        <td style="background: linear-gradient(135deg, {} 0%, {} 100%); padding: 30px; text-align: center;">
                            {}
                            <h1 style="color: #ffffff; margin: 10px 0 0 0; font-size: 24px; font-weight: 600;">{}</h1>
                        </td>
                    </tr>
                    <!-- Content -->
                    <tr>
                        <td style="padding: 40px 30px;">
                            {}
                        </td>
                    </tr>
                    <!-- Footer -->
                    <tr>
                        <td style="background-color: #f8f9fa; padding: 20px 30px; text-align: center; border-top: 1px solid #e9ecef;">
                            <p style="color: #6c757d; font-size: 12px; margin: 0 0 5px 0;">
                                This email was sent by {}. If you didn't request this, please ignore it.
                            </p>
                            <p style="color: #adb5bd; font-size: 11px; margin: 0;">
                                &copy; {} All rights reserved.
                            </p>
                        </td>
                    </tr>
                </table>
            </td>
        </tr>
    </table>
</body>
</html>"#,
        config.app_name,
        header_text,
        config.primary_color,
        config.secondary_color,
        logo_html,
        header_text,
        content,
        config.app_name,
        config.app_name
    )
}

/// Password Reset Email Template
pub mod password_reset {
    use super::*;

    /// Build a password reset email
    pub fn build(config: &EmailTemplateConfig, reset_link: &str) -> Email {
        let content = format!(
            r#"<p style="margin: 0 0 20px 0; font-size: 16px;">Hello,</p>
<p style="margin: 0 0 20px 0; font-size: 16px;">We received a request to reset your password. Click the button below to create a new password:</p>
<table width="100%" cellpadding="0" cellspacing="0">
    <tr>
        <td align="center" style="padding: 20px 0;">
            <a href="{}" style="background: {}; color: #ffffff; padding: 14px 32px; text-decoration: none; border-radius: 6px; font-weight: 600; font-size: 16px; display: inline-block;">Reset Password</a>
        </td>
    </tr>
</table>
<p style="margin: 0 0 20px 0; font-size: 14px; color: #666;">This link will expire in 1 hour.</p>
<p style="margin: 0 0 20px 0; font-size: 14px; color: #666;">If you didn't request this password reset, please ignore this email. Your password will remain unchanged.</p>
<hr style="border: none; border-top: 1px solid #e9ecef; margin: 25px 0;">
<p style="margin: 0; font-size: 12px; color: #999;">If the button doesn't work, copy and paste this link into your browser:<br><a href="{}" style="color: {}; text-decoration: underline;">{}</a></p>"#,
            reset_link, config.primary_color, reset_link, config.primary_color, reset_link
        );

        let html = build_html_wrapper(config, "Password Reset Request", &content);

        Email::new(
            "noreply@example.com",
            "placeholder@example.com",
            "Password Reset Request",
        )
        .html(html)
    }
}

/// Welcome Email Template
pub mod welcome {
    use super::*;

    /// Build a welcome email
    pub fn build(config: &EmailTemplateConfig, name: &str, verify_link: Option<&str>) -> Email {
        let verify_section = verify_link.map_or(String::new(), |link| {
            format!(
                r#"<table width="100%" cellpadding="0" cellspacing="0" style="margin: 20px 0;">
    <tr>
        <td align="center">
            <a href="{}" style="background: {}; color: #ffffff; padding: 14px 32px; text-decoration: none; border-radius: 6px; font-weight: 600; font-size: 16px; display: inline-block;">Verify Email</a>
        </td>
    </tr>
</table>
<p style="margin: 0 0 20px 0; font-size: 14px; color: #666;">This link will expire in 24 hours.</p>"#,
                link,
                config.primary_color
            )
        });

        let content = format!(
            r#"<p style="margin: 0 0 20px 0; font-size: 16px;">Hello{},</p>
<p style="margin: 0 0 20px 0; font-size: 16px;">Welcome to {}! We're excited to have you on board.</p>
<p style="margin: 0 0 20px 0; font-size: 16px;">To get started, please verify your email address:</p>
{}"#,
            if name.is_empty() {
                String::new()
            } else {
                format!(" {}", name)
            },
            config.app_name,
            verify_section
        );

        let html = build_html_wrapper(config, "Welcome!", &content);

        Email::new("noreply@example.com", "placeholder@example.com", "Welcome!").html(html)
    }
}

/// Email Verification Template
pub mod verify_email {
    use super::*;

    /// Build an email verification email
    pub fn build(config: &EmailTemplateConfig, verify_link: &str, code: Option<&str>) -> Email {
        let code_section = code.map_or(String::new(), |c| {
            format!(
                r#"<div style="background: #f8f9fa; padding: 15px; border-radius: 6px; text-align: center; margin: 20px 0;">
    <p style="margin: 0 0 10px 0; font-size: 14px; color: #666;">Or use this verification code:</p>
    <p style="margin: 0; font-size: 24px; font-weight: bold; letter-spacing: 4px; color: {};">{}</p>
</div>"#,
                config.primary_color,
                c
            )
        });

        let content = format!(
            r#"<p style="margin: 0 0 20px 0; font-size: 16px;">Hello,</p>
<p style="margin: 0 0 20px 0; font-size: 16px;">Please verify your email address by clicking the button below:</p>
<table width="100%" cellpadding="0" cellspacing="0" style="margin: 20px 0;">
    <tr>
        <td align="center">
            <a href="{}" style="background: {}; color: #ffffff; padding: 14px 32px; text-decoration: none; border-radius: 6px; font-weight: 600; font-size: 16px; display: inline-block;">Verify Email</a>
        </td>
    </tr>
</table>
{}
<p style="margin: 0 0 20px 0; font-size: 14px; color: #666;">This link will expire in 24 hours.</p>"#,
            verify_link, config.primary_color, code_section
        );

        let html = build_html_wrapper(config, "Verify Your Email", &content);

        Email::new(
            "noreply@example.com",
            "placeholder@example.com",
            "Verify Your Email",
        )
        .html(html)
    }
}

/// Magic Link Login Template
pub mod magic_link {
    use super::*;

    /// Build a magic link login email
    pub fn build(config: &EmailTemplateConfig, magic_link: &str, code: Option<&str>) -> Email {
        let code_section = code.map_or(String::new(), |c| {
            format!(
                r#"<div style="background: #f8f9fa; padding: 15px; border-radius: 6px; text-align: center; margin: 20px 0;">
    <p style="margin: 0 0 10px 0; font-size: 14px; color: #666;">Or use this magic code:</p>
    <p style="margin: 0; font-size: 28px; font-weight: bold; letter-spacing: 6px; color: {};">{}</p>
</div>"#,
                config.primary_color,
                c
            )
        });

        let content = format!(
            r#"<p style="margin: 0 0 20px 0; font-size: 16px;">Hello,</p>
<p style="margin: 0 0 20px 0; font-size: 16px;">Use the button below to log in to your account instantly - no password needed!</p>
<table width="100%" cellpadding="0" cellspacing="0" style="margin: 20px 0;">
    <tr>
        <td align="center">
            <a href="{}" style="background: {}; color: #ffffff; padding: 14px 32px; text-decoration: none; border-radius: 6px; font-weight: 600; font-size: 16px; display: inline-block;">Log In</a>
        </td>
    </tr>
</table>
{}
<p style="margin: 0 0 20px 0; font-size: 14px; color: #666;">This link will expire in 15 minutes.</p>
<p style="margin: 0; font-size: 14px; color: #666;">If you didn't request this magic link, you can safely ignore this email.</p>"#,
            magic_link, config.primary_color, code_section
        );

        let html = build_html_wrapper(config, "Your Magic Login Link", &content);

        Email::new(
            "noreply@example.com",
            "placeholder@example.com",
            "Your Magic Login Link",
        )
        .html(html)
    }
}

/// Generic Notification Template
pub mod notification {
    use super::*;

    /// Build a generic notification email
    pub fn build(
        config: &EmailTemplateConfig,
        title: &str,
        message: &str,
        action_text: Option<&str>,
        action_link: Option<&str>,
    ) -> Email {
        let action_section = match (action_text, action_link) {
            (Some(text), Some(link)) => format!(
                r#"<table width="100%" cellpadding="0" cellspacing="0" style="margin: 20px 0;">
    <tr>
        <td align="center">
            <a href="{}" style="background: {}; color: #ffffff; padding: 12px 24px; text-decoration: none; border-radius: 6px; font-weight: 600; font-size: 14px; display: inline-block;">{}</a>
        </td>
    </tr>
</table>"#,
                link, config.primary_color, text
            ),
            _ => String::new(),
        };

        let content = format!(
            r#"<h2 style="margin: 0 0 15px 0; font-size: 20px; color: #333;">{}</h2>
<p style="margin: 0 0 20px 0; font-size: 16px; color: #555;">{}</p>
{}"#,
            title, message, action_section
        );

        let html = build_html_wrapper(config, title, &content);

        Email::new("noreply@example.com", "placeholder@example.com", title).html(html)
    }
}

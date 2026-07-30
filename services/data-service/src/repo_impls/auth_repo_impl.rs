use domain::models::user::User;
use crate::entities::user;

pub struct AuthUserView(pub user::Model);

impl From<AuthUserView> for identity_shapes::AuthUser {
    fn from(v: AuthUserView) -> Self {
        let m = v.0;
        identity_shapes::AuthUser {
            base: User { uuid: m.uuid, pan: m.pan, name: m.name },
            password_hash: m.password,
            email: m.email,
            mobile: m.mobile,
            login_flag: m.login_flag != 0,
        }
    }
}
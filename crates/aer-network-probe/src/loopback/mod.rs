pub(crate) mod fixture;
pub(crate) mod record;
pub(crate) mod sampler;
pub(crate) mod supervisor;

use fixture::FixtureFamily;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Role {
    Supervisor,
    Sampler,
    Fixture(FixtureFamily),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RunError {
    Arguments,
    Fixture,
    Sampler,
    Supervisor,
}

pub(crate) fn parse_role(arguments: &[String]) -> Result<Role, RunError> {
    match arguments {
        [role] if role == "supervisor" => Ok(Role::Supervisor),
        [role] if role == "sampler" => Ok(Role::Sampler),
        [role, family] if role == "fixture" && family == "ipv4" => {
            Ok(Role::Fixture(FixtureFamily::Ipv4))
        }
        [role, family] if role == "fixture" && family == "ipv6" => {
            Ok(Role::Fixture(FixtureFamily::Ipv6))
        }
        _ => Err(RunError::Arguments),
    }
}

pub(crate) fn run(role: Role) -> Result<(), RunError> {
    match role {
        Role::Supervisor => supervisor::run().map_err(|_| RunError::Supervisor),
        Role::Sampler => sampler::run().map_err(|_| RunError::Sampler),
        Role::Fixture(family) => fixture::run(family).map_err(|_| RunError::Fixture),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    #[test]
    fn roles_accept_only_the_closed_argument_grammar() {
        assert_eq!(parse_role(&args(&["supervisor"])), Ok(Role::Supervisor));
        assert_eq!(parse_role(&args(&["sampler"])), Ok(Role::Sampler));
        assert_eq!(
            parse_role(&args(&["fixture", "ipv4"])),
            Ok(Role::Fixture(FixtureFamily::Ipv4))
        );
        assert_eq!(
            parse_role(&args(&["fixture", "ipv6"])),
            Ok(Role::Fixture(FixtureFamily::Ipv6))
        );
        for invalid in [
            args(&[]),
            args(&["fixture"]),
            args(&["fixture", "external"]),
            args(&["supervisor", "extra"]),
            args(&["unknown"]),
        ] {
            assert_eq!(parse_role(&invalid), Err(RunError::Arguments));
        }
    }
}

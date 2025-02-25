use ansiterm::Style;
use uzers::{Groups, Users};

use crate::fs::fields as f;
use crate::output::cell::TextCell;
use crate::output::table::UserFormat;

pub trait Render {
    fn render<C: Colours, U: Users + Groups>(
        self,
        colours: &C,
        users: &U,
        format: UserFormat,
    ) -> TextCell;
}

impl Render for Option<f::Group> {
    fn render<C: Colours, U: Users + Groups>(
        self,
        colours: &C,
        users: &U,
        format: UserFormat,
    ) -> TextCell {
        use uzers::os::unix::GroupExt;

        let mut style = colours.not_yours();

        let group = match self {
            Some(g) => match users.get_group_by_gid(g.0) {
                Some(g) => (*g).clone(),
                None => return TextCell::paint(style, g.0.to_string()),
            },
            None => return TextCell::blank(colours.no_group()),
        };

        let current_uid = users.get_current_uid();
        if let Some(current_user) = users.get_user_by_uid(current_uid) {
            if current_user.primary_group_id() == group.gid()
                || group.members().iter().any(|u| u == current_user.name())
            {
                style = colours.yours();
            }
        }

        if group.gid() == 0 {
            style = colours.root_group();
        }

        let group_name = match format {
            UserFormat::Name => group.name().to_string_lossy().into(),
            UserFormat::Numeric => group.gid().to_string(),
        };

        TextCell::paint(style, group_name)
    }
}

pub trait Colours {
    fn yours(&self) -> Style;
    fn not_yours(&self) -> Style;
    fn no_group(&self) -> Style;
    fn root_group(&self) -> Style;
}

#[cfg(test)]
#[allow(unused_results)]
pub mod test {
    use super::{Colours, Render};
    use crate::fs::fields as f;
    use crate::output::cell::TextCell;
    use crate::output::table::UserFormat;

    use ansiterm::Colour::*;
    use ansiterm::Style;
    use uzers::mock::MockUsers;
    use uzers::os::unix::GroupExt;
    use uzers::{Group, User};

    struct TestColours;

    #[rustfmt::skip]
    impl Colours for TestColours {
        fn yours(&self)     -> Style { Fixed(80).normal() }
        fn not_yours(&self) -> Style { Fixed(81).normal() }
        fn no_group(&self)   -> Style { Black.italic() }
        fn root_group(&self) -> Style { Fixed(82).normal() }
    }

    #[test]
    fn named() {
        let mut users = MockUsers::with_current_uid(1000);
        users.add_group(Group::new(100, "folk"));

        let group = Some(f::Group(100));
        let expected = TextCell::paint_str(Fixed(81).normal(), "folk");
        assert_eq!(
            expected,
            group.render(&TestColours, &users, UserFormat::Name)
        );

        let expected = TextCell::paint_str(Fixed(81).normal(), "100");
        assert_eq!(
            expected,
            group.render(&TestColours, &users, UserFormat::Numeric)
        );
    }

    #[test]
    fn unnamed() {
        let users = MockUsers::with_current_uid(1000);

        let group = Some(f::Group(100));
        let expected = TextCell::paint_str(Fixed(81).normal(), "100");
        assert_eq!(
            expected,
            group.render(&TestColours, &users, UserFormat::Name)
        );
        assert_eq!(
            expected,
            group.render(&TestColours, &users, UserFormat::Numeric)
        );
    }

    #[test]
    fn primary() {
        let mut users = MockUsers::with_current_uid(2);
        users.add_user(User::new(2, "eve", 100));
        users.add_group(Group::new(100, "folk"));

        let group = Some(f::Group(100));
        let expected = TextCell::paint_str(Fixed(80).normal(), "folk");
        assert_eq!(
            expected,
            group.render(&TestColours, &users, UserFormat::Name)
        )
    }

    #[test]
    fn secondary() {
        let mut users = MockUsers::with_current_uid(2);
        users.add_user(User::new(2, "eve", 666));

        let test_group = Group::new(100, "folk").add_member("eve");
        users.add_group(test_group);

        let group = Some(f::Group(100));
        let expected = TextCell::paint_str(Fixed(80).normal(), "folk");
        assert_eq!(
            expected,
            group.render(&TestColours, &users, UserFormat::Name)
        )
    }

    #[test]
    fn overflow() {
        let group = Some(f::Group(2_147_483_648));
        let expected = TextCell::paint_str(Fixed(81).normal(), "2147483648");
        assert_eq!(
            expected,
            group.render(
                &TestColours,
                &MockUsers::with_current_uid(0),
                UserFormat::Numeric
            )
        );
    }
}

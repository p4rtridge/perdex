use std::sync::LazyLock;

use bubble_bath::BubbleBath;

use crate::ap_type::actor::Actor;

pub trait SanitizeExt {
    fn clean_html(&mut self);
}

impl SanitizeExt for Actor {
    fn clean_html(&mut self) {
        if let Some(ref mut name) = self.name {
            name.clean_html();
        }
    }
}

impl SanitizeExt for String {
    fn clean_html(&mut self) {
        static BUBBLE_BATH: LazyLock<BubbleBath<'static>> = LazyLock::new(|| BubbleBath {
            preserve_escaped: true,
            ..BubbleBath::default()
        });

        *self = BUBBLE_BATH.clean(self).unwrap();
    }
}

use crate::graphics::{Area, Corners};
use crate::{Radius, Rectangle};

const MIXED: &str = "radius and radius_<corner> can't be mixed on one Rectangle";

// one way of rounding at a time, so radius and the radius_ corner methods can't both be set
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Rounding {
    Square,

    // every corner alike, from radius
    Same(Radius),

    // each corner apart, from the radius_ corner methods
    Each(EachCorner),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EachCorner {
    pub top_left: Radius,
    pub top_right: Radius,
    pub bottom_right: Radius,
    pub bottom_left: Radius,
}

impl Rectangle {
    // every corner alike; can't be mixed with the radius_ corner methods
    pub fn radius(mut self, radius: impl Into<Radius>) -> Self {
        let mixed = matches!(self.rounding, Rounding::Each(_));

        assert!(!mixed, "{MIXED}");

        self.rounding = Rounding::Same(radius.into());

        self
    }

    pub fn radius_top_left(mut self, radius: impl Into<Radius>) -> Self {
        self.each_corner().top_left = radius.into();

        self
    }

    pub fn radius_top_right(mut self, radius: impl Into<Radius>) -> Self {
        self.each_corner().top_right = radius.into();

        self
    }

    pub fn radius_bottom_right(mut self, radius: impl Into<Radius>) -> Self {
        self.each_corner().bottom_right = radius.into();

        self
    }

    pub fn radius_bottom_left(mut self, radius: impl Into<Radius>) -> Self {
        self.each_corner().bottom_left = radius.into();

        self
    }

    // switches to rounding each corner apart; corners left unset stay square
    fn each_corner(&mut self) -> &mut EachCorner {
        let mixed = matches!(self.rounding, Rounding::Same(_));

        assert!(!mixed, "{MIXED}");

        if matches!(self.rounding, Rounding::Square) {
            let square = Radius::Fixed(0.0);

            self.rounding = Rounding::Each(EachCorner {
                top_left: square,
                top_right: square,
                bottom_right: square,
                bottom_left: square,
            });
        }

        let Rounding::Each(corners) = &mut self.rounding else {
            unreachable!("rounding was just set to each corner");
        };

        corners
    }
}

// the corners in pixels for this area, where Full is half the shorter side
pub fn resolve(rounding: Rounding, area: Area) -> Corners {
    let to_pixels = |radius: Radius| radius.resolve(area.width, area.height);

    match rounding {
        Rounding::Square => Corners::from(0.0),

        Rounding::Same(radius) => Corners::from(to_pixels(radius)),

        Rounding::Each(each) => Corners {
            top_left: to_pixels(each.top_left),
            top_right: to_pixels(each.top_right),
            bottom_right: to_pixels(each.bottom_right),
            bottom_left: to_pixels(each.bottom_left),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Full;

    #[test]
    fn corners_set_apart_from_radius() {
        let rectangle = Rectangle::new()
            .width(10.0)
            .height(10.0)
            .radius_top_left(Full);

        let square = Radius::Fixed(0.0);

        let expected = Rounding::Each(EachCorner {
            top_left: Radius::Full,
            top_right: square,
            bottom_right: square,
            bottom_left: square,
        });

        assert_eq!(rectangle.rounding, expected);
    }

    #[test]
    #[should_panic(expected = "can't be mixed")]
    fn mixing_radius_and_corners_panics() {
        let _ = Rectangle::new()
            .width(10.0)
            .height(10.0)
            .radius(Full)
            .radius_top_left(4.0);
    }
}

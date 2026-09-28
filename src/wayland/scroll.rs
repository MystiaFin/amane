use smithay_client_toolkit::seat::pointer::AxisScroll;

use crate::Scroll;

// how many pixels a compositor usually reports for one wheel step
const PIXELS_PER_LINE: f64 = 15.0;

pub fn translate(horizontal: &AxisScroll, vertical: &AxisScroll) -> Scroll {
    Scroll {
        x: lines(horizontal),
        y: lines(vertical),
    }
}

fn lines(axis: &AxisScroll) -> f32 {
    // a wheel reports its steps in 120ths, which is exact
    if axis.value120 != 0 {
        return axis.value120 as f32 / 120.0;
    }

    // older compositors report whole steps instead
    if axis.discrete != 0 {
        return axis.discrete as f32;
    }

    // a touchpad only reports pixels
    let lines = axis.absolute / PIXELS_PER_LINE;

    lines as f32
}

// Not the smartest, but I guess it'll work
pub fn next_nearest(current: u32, multiple: u32) -> u32 {
    if current % multiple == 0 {
        current
    } else {
        next_nearest(current + 1, multiple)
    }
}

/*
pub fn highest(target: u32) -> u32 {}

fn _highest(target: u32, highest_mutliple: u32, current_mutliple: u32) -> u32 {
    let remainder = target % current_mutliple;
    let has_reached_halfway = target / 2 == current_mutliple;

}
 */

#[cfg(test)]
mod tests {
    use super::*;

    mod next_nearest {
        use super::*;

        #[test]
        fn finds_next_multiple_when_current_isnt_multiple() {
            for (current, multiple, expected) in [(9, 2, 10), (253, 4, 256), (10, 3, 12)] {
                let actual = next_nearest(current, multiple);
                assert_eq!(actual, expected);
            }
        }

        #[test]
        fn finds_next_multiple_when_current_is_multiple() {
            for (current, multiple) in [(10, 5), (9, 3)] {
                let actual = next_nearest(current, multiple);
                assert_eq!(actual, current);
            }
        }
    }
}

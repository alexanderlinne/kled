pub mod local;
pub mod shared;

pub(crate) fn distribute_value<T, F, Value>(vec: &mut Vec<T>, f: F, value: Value)
where
    F: Fn(&mut T, Value),
    Value: Copy,
{
    match vec.len() {
        0 => (),
        1 => f(&mut vec[0], value),
        len => {
            for t in vec.iter_mut().take(len - 1) {
                f(t, value.clone());
            }
            f(&mut vec[len - 1], value);
        }
    }
}

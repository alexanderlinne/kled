pub mod local;
pub mod shared;

pub(crate) fn distribute_value<T, F, Value>(vec: &mut Vec<T>, f: F, value: Value)
where
    F: Fn(&mut T, Value),
    Value: Copy,
{
    let len = vec.len();
    for t in vec.iter_mut().take(len - 1) {
        f(t, value.clone());
    }
    f(&mut vec[len - 1], value);
}

use std::fmt::{Display, Formatter, Result};
use std::ops::Add;



#[derive(Default, PartialEq, Debug, Clone, Copy)]
struct Complex<T> {
    re: T,
    im: T,
}

#[allow(dead_code)]
impl<T> Complex<T> {
    fn new(re: T, im: T) -> Self {
        Complex::<T> { re, im }
    }
}

impl<T: Add<T, Output = T>> Add for Complex<T> {
    type Output = Complex<T>;

    fn add(self, rhs: Complex<T>) -> Self::Output {
        Complex::<T> {
            re: self.re + rhs.re,
            im: self.im + rhs.im,
        }
    }
}

impl<T> From<(T, T)> for Complex<T> {
    fn from(value: (T, T)) -> Complex<T> {
        Complex::<T> {
            re: value.0,
            im: value.1,
        }
    }
}

impl<T: Display> Display for Complex<T> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{} + {}i", self.re, self.im)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_complex() {
        let first = Complex::new(4, 6);
        let second: Complex<i32> = Complex::default();
        assert_eq!(first.re + first.im, 10);
        assert_eq!(second.re + second.im, 0);
        assert_ne!(first, second);
    }

    #[test]
    fn complex_addition() {
        let a = Complex::new(1, -2);
        let b = Complex::default();
        assert_eq!(a + b, a);
    }

    #[test]
    fn complex_from() {
        let a = (123, 234);
        let complex = Complex::from(a);

        assert_eq!(complex.re, 123);
        assert_eq!(complex.im, 234);
    }

    #[test]
    fn complex_deiplay() {
        let my_imagination = Complex::new(123, 234);
        println!("{}", my_imagination);
    }
}

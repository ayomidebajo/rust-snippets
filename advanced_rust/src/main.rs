fn main() {
    let mut num = 5;

    let r1 = &num as *const i32;
    let r2 = &mut num as *mut i32;

    unsafe {
        println!("r1 is: {}", *r1);
        println!("r2 is: {}", *r2);
    }

    unsafe fn dangerous() {}

    fn safe() {}

    unsafe {
        safe();
        dangerous()
    }

    // let x = 4;
    // let y = 5;
    // let z = x - y;
    // let y = 3;

    // println!("z: {}", z);
    // do {}; while x > 0;
    // for _ in 1..2 {}
    // while x > 0 {}
    // loop {}
    // for _ in vec![1, 2, 3] {}
    let n = 5;

    // (1..=n).step_by(1);

    // fn sort<T>(items: &mut [T]){

    // }
// #[derive(Debug)]
//     struct Fraction {
// 	numerator: u32,
// 	denominator: u32,
//     }


//      impl From<u32> for Fraction {
//  	fn from(n: u32) -> Self {
// 		Self {
// 			numerator: n,
// 		denominator: 1,
// 		}
// 	}
//  }

//  fn mains() {
//  	let a: u32 = 5;
//  	let b: Fraction = a.into();
//     println!("b : {:?}", b);
//  }



//  mains();

use std::num::ParseIntError;
///
enum OutOfRangeError {
 	TooLarge,
 	TooSmall,
 	NotEvenANumber,
 }
 
 impl From<ParseIntError> for OutOfRangeError {
 	fn from(_e: ParseIntError) -> Self {
 		Self::NotEvenANumber
 	}
 }

//  learn more advanced traits , types, macros and algorithms

 fn string_to_int_in_range(s: String) -> Result<u32, OutOfRangeError> {
 	// Given: The u32::from_str_radix function returns Result<u32, ParseIntError>
 	let n: u32 = u32::from_str_radix(&s,10)?;

 	match n {
 		n if n < 5 => Err(OutOfRangeError::TooSmall),
 		n if n > 100 => Err(OutOfRangeError::TooLarge),
 		n => Ok(n),
 	}
}

    let mut v = vec![1, 2, 3, 4, 5, 6, 7];

    let r = &mut v[..];

    let (a, b) = r.split_at_mut(3);

    assert_eq!(a, &mut [1, 2, 3]);
    assert_eq!(b, &mut [4, 5, 6, 7]);
}

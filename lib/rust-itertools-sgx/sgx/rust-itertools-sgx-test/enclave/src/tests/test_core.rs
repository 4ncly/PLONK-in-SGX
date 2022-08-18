//! Licensed under the Apache License, Version 2.0
//! http://www.apache.org/licenses/LICENSE-2.0 or the MIT license
//! http://opensource.org/licenses/MIT, at your
//! option. This file may not be copied, modified, or distributed
//! except according to those terms.
//#![no_std]

use core::iter;

use itertools::Itertools;
use itertools::interleave;
use itertools::multizip;
use itertools::free::put_back;

//#[test]
pub fn product2() {
    let s = "αβ";

    let mut prod = iproduct!(s.chars(), 0..2);
    assert!(prod.next() == Some(('α', 0)));
    assert!(prod.next() == Some(('α', 1)));
    assert!(prod.next() == Some(('β', 0)));
    assert!(prod.next() == Some(('β', 1)));
    assert!(prod.next() == None);
}

//#[test]
pub fn product_temporary() {
    for (_x, _y, _z) in iproduct!(
        [0, 1, 2].iter().cloned(),
        [0, 1, 2].iter().cloned(),
        [0, 1, 2].iter().cloned())
    {
        // ok
    }
}


//#[test]
pub fn izip_macro() {
    let mut zip = izip!(2..3);
    assert!(zip.next() == Some(2));
    assert!(zip.next().is_none());

    let mut zip = izip!(0..3, 0..2, 0..2i8);
    for i in 0..2 {
        assert!((i as usize, i, i as i8) == zip.next().unwrap());
    }
    assert!(zip.next().is_none());

    let xs: [isize; 0] = [];
    let mut zip = izip!(0..3, 0..2, 0..2i8, &xs);
    assert!(zip.next().is_none());
}

//#[test]
pub fn izip2() {
    let _zip1: iter::Zip<_, _> = izip!(1.., 2..);
    let _zip2: iter::Zip<_, _> = izip!(1.., 2.., );
}

//#[test]
pub fn izip3() {
    let mut zip: iter::Map<iter::Zip<_, _>, _> = izip!(0..3, 0..2, 0..2i8);
    for i in 0..2 {
        assert!((i as usize, i, i as i8) == zip.next().unwrap());
    }
    assert!(zip.next().is_none());
}

//#[test]
pub fn multizip3() {
    let mut zip = multizip((0..3, 0..2, 0..2i8));
    for i in 0..2 {
        assert!((i as usize, i, i as i8) == zip.next().unwrap());
    }
    assert!(zip.next().is_none());

    let xs: [isize; 0] = [];
    let mut zip = multizip((0..3, 0..2, 0..2i8, xs.iter()));
    assert!(zip.next().is_none());

    for (_, _, _, _, _) in multizip((0..3, 0..2, xs.iter(), &xs, xs.to_vec())) {
        /* test compiles */
    }
}

//#[test]
pub fn write_to() {
    let xs = [7, 9, 8];
    let mut ys = [0; 5];
    let cnt = ys.iter_mut().set_from(xs.iter().map(|x| *x));
    assert!(cnt == xs.len());
    assert!(ys == [7, 9, 8, 0, 0]);

    let cnt = ys.iter_mut().set_from(0..10);
    assert!(cnt == ys.len());
    assert!(ys == [0, 1, 2, 3, 4]);
}

//#[test]
pub fn test_interleave() {
    let xs: [u8; 0]  = [];
    let ys = [7u8, 9, 8, 10];
    let zs = [2u8, 77];
    let it = interleave(xs.iter(), ys.iter());
    itertools::assert_equal(it, ys.iter());

    let rs = [7u8, 2, 9, 77, 8, 10];
    let it = interleave(ys.iter(), zs.iter());
    itertools::assert_equal(it, rs.iter());
}

#[allow(deprecated)]
//#[test]
pub fn foreach() {
    let xs = [1i32, 2, 3];
    let mut sum = 0;
    xs.iter().foreach(|elt| sum += *elt);
    assert!(sum == 6);
}

//#[test]
pub fn dropping() {
    let xs = [1, 2, 3];
    let mut it = xs.iter().dropping(2);
    assert_eq!(it.next(), Some(&3));
    assert!(it.next().is_none());
    let mut it = xs.iter().dropping(5);
    assert!(it.next().is_none());
}

//#[test]
pub fn batching() {
    let xs = [0, 1, 2, 1, 3];
    let ys = [(0, 1), (2, 1)];

    // An iterator that gathers elements up in pairs
    let pit = xs.iter().cloned().batching(|it| {
               match it.next() {
                   None => None,
                   Some(x) => match it.next() {
                       None => None,
                       Some(y) => Some((x, y)),
                   }
               }
           });
    itertools::assert_equal(pit, ys.iter().cloned());
}

//#[test]
pub fn test_put_back() {
    let xs = [0, 1, 1, 1, 2, 1, 3, 3];
    let mut pb = put_back(xs.iter().cloned());
    pb.next();
    pb.put_back(1);
    pb.put_back(0);
    itertools::assert_equal(pb, xs.iter().cloned());
}

#[allow(deprecated)]
//#[test]
pub fn step() {
    itertools::assert_equal((0..10).step(1), 0..10);
    itertools::assert_equal((0..10).step(2), (0..10).filter(|x: &i32| *x % 2 == 0));
    itertools::assert_equal((0..10).step(10), 0..1);
}

#[allow(deprecated)]
//#[test]
pub fn merge() {
    itertools::assert_equal((0..10).step(2).merge((1..10).step(2)), 0..10);
}


//#[test]
pub fn repeatn() {
    let s = "α";
    let mut it = itertools::repeat_n(s, 3);
    assert_eq!(it.len(), 3);
    assert_eq!(it.next(), Some(s));
    assert_eq!(it.next(), Some(s));
    assert_eq!(it.next(), Some(s));
    assert_eq!(it.next(), None);
    assert_eq!(it.next(), None);
}

//#[test]
pub fn count_clones() {
    // Check that RepeatN only clones N - 1 times.

    use core::cell::Cell;
    #[derive(PartialEq, Debug)]
    struct Foo {
        n: Cell<usize>
    }

    impl Clone for Foo
    {
        fn clone(&self) -> Self
        {
            let n = self.n.get();
            self.n.set(n + 1);
            Foo { n: Cell::new(n + 1) }
        }
    }


    for n in 0..10 {
        let f = Foo{n: Cell::new(0)};
        let it = itertools::repeat_n(f, n);
        // drain it
        let last = it.last();
        if n == 0 {
            assert_eq!(last, None);
        } else {
            assert_eq!(last, Some(Foo{n: Cell::new(n - 1)}));
        }
    }
}

//#[test]
pub fn part() {
    let mut data = [7, 1, 1, 9, 1, 1, 3];
    let i = itertools::partition(&mut data, |elt| *elt >= 3);
    assert_eq!(i, 3);
    assert_eq!(data, [7, 3, 9, 1, 1, 1, 1]);

    let i = itertools::partition(&mut data, |elt| *elt == 1);
    assert_eq!(i, 4);
    assert_eq!(data, [1, 1, 1, 1, 9, 3, 7]);

    let mut data = [1, 2, 3, 4, 5, 6, 7, 8, 9];
    let i = itertools::partition(&mut data, |elt| *elt % 3 == 0);
    assert_eq!(i, 3);
    assert_eq!(data, [9, 6, 3, 4, 5, 2, 7, 8, 1]);
}

//#[test]
pub fn tree_fold1() {
    for i in 0..100 {
        assert_eq!((0..i).tree_fold1(|x, y| x + y), (0..i).fold1(|x, y| x + y));
    }
}

//#[test]
pub fn exactly_one() {
    assert_eq!((0..10).filter(|&x| x == 2).exactly_one().unwrap(), 2);
    assert!((0..10).filter(|&x| x > 1 && x < 4).exactly_one().unwrap_err().eq(2..4));
    assert!((0..10).filter(|&x| x > 1 && x < 5).exactly_one().unwrap_err().eq(2..5));
    assert!((0..10).filter(|&_| false).exactly_one().unwrap_err().eq(0..0));
}

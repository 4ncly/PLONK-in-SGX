// Copyright (C) 2017-2018 Baidu, Inc. All Rights Reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions
// are met:
//
//  * Redistributions of source code must retain the above copyright
//    notice, this list of conditions and the following disclaimer.
//  * Redistributions in binary form must reproduce the above copyright
//    notice, this list of conditions and the following disclaimer in
//    the documentation and/or other materials provided with the
//    distribution.
//  * Neither the name of Baidu, Inc., nor the names of its
//    contributors may be used to endorse or promote products derived
//    from this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

#![crate_name = "helloworldsampleenclave"]
#![crate_type = "staticlib"]

#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

extern crate sgx_types;
#[cfg(not(target_env = "sgx"))]
#[macro_use]
extern crate sgx_tstd as std;
extern crate sgx_tunittest;

#[cfg(target_env = "sgx")]
extern crate core;

#[macro_use]
extern crate itertools;
extern crate rand;
extern crate quickcheck;
//extern crate permutohedron;

use sgx_types::*;
use crate::std::string::String;
use crate::std::vec::Vec;
use crate::std::io::{self, Write};
use crate::std::slice;
use crate::std::panic;
use sgx_tunittest::*;

mod tests;

#[no_mangle]
pub extern "C" fn say_something(some_string: *const u8, some_len: usize) -> sgx_status_t {

    let str_slice = unsafe { slice::from_raw_parts(some_string, some_len) };
    let _ = io::stdout().write(str_slice);

    // A sample &'static string
    let rust_raw_string = "This is a in-Enclave ";
    // An array
    let word:[u8;4] = [82, 117, 115, 116];
    // An vector
    let word_vec:Vec<u8> = vec![32, 115, 116, 114, 105, 110, 103, 33];

    // Construct a string from &'static string
    let mut hello_string = String::from(rust_raw_string);

    // Iterate on word array
    for c in word.iter() {
        hello_string.push(*c as char);
    }

    // Rust style convertion
    hello_string += String::from_utf8(word_vec).expect("Invalid UTF-8")
                                               .as_str();

    // Ocall to normal world for output
    println!("{}", &hello_string);

    rsgx_unit_tests!(
tests::merge_join::empty,
tests::merge_join::left_only,
tests::merge_join::right_only,
tests::merge_join::first_left_then_right,
tests::merge_join::first_right_then_left,
tests::merge_join::interspersed_left_and_right,
tests::merge_join::overlapping_left_and_right,
tests::peeking_take_while::peeking_take_while_peekable,
tests::peeking_take_while::peeking_take_while_put_back,
tests::peeking_take_while::peeking_take_while_put_back_n,
tests::peeking_take_while::peeking_take_while_slice_iter,
tests::peeking_take_while::peeking_take_while_slice_iter_rev,
tests::quick::size_product,
tests::quick::size_product3,
tests::quick::correct_cartesian_product3,
tests::quick::size_multi_product,
tests::quick::correct_multi_product3,
tests::quick::size_step,
tests::quick::equal_step,
tests::quick::equal_step_vec,
tests::quick::size_multipeek,
tests::quick::equal_merge,
tests::quick::size_merge,
tests::quick::size_zip,
tests::quick::size_zip_rc,
tests::quick::size_zip_macro,
tests::quick::equal_kmerge,
tests::quick::equal_kmerge_2,
tests::quick::equal_kmerge_by_ge,
tests::quick::equal_kmerge_by_lt,
tests::quick::equal_kmerge_by_le,
tests::quick::size_kmerge,
tests::quick::equal_zip_eq,
tests::quick::size_zip_longest,
tests::quick::size_2_zip_longest,
tests::quick::size_interleave,
tests::quick::exact_interleave,
tests::quick::size_interleave_shortest,
tests::quick::exact_interleave_shortest,
tests::quick::size_intersperse,
tests::quick::equal_intersperse,
tests::quick::equal_combinations_2,
tests::quick::collect_tuple_matches_size,
tests::quick::equal_dedup,
tests::quick::size_dedup,
tests::quick::exact_repeatn,
tests::quick::size_put_back,
tests::quick::size_put_backn,
tests::quick::size_tee,
tests::quick::size_tee_2,
tests::quick::size_take_while_ref,
tests::quick::equal_partition,
tests::quick::size_combinations,
tests::quick::equal_combinations,
tests::quick::size_pad_tail,
tests::quick::size_pad_tail2,
tests::quick::size_unique,
tests::quick::count_unique,
tests::quick::fuzz_group_by_lazy_1,
tests::quick::fuzz_group_by_lazy_2,
tests::quick::fuzz_group_by_lazy_3,
tests::quick::fuzz_group_by_lazy_duo,
tests::quick::equal_chunks_lazy,
tests::quick::equal_tuple_windows_1,
tests::quick::equal_tuple_windows_2,
tests::quick::equal_tuple_windows_3,
tests::quick::equal_tuple_windows_4,
tests::quick::equal_tuples_1,
tests::quick::equal_tuples_2,
tests::quick::equal_tuples_3,
tests::quick::equal_tuples_4,
tests::quick::exact_tuple_buffer,
tests::quick::with_position_exact_size_1,
tests::quick::with_position_exact_size_2,
tests::quick::correct_group_map_modulo_key,
tests::quick::minmax,
tests::quick::minmax_f64,
tests::quick::tree_fold1_f64,
tests::quick::exactly_one_i32,
tests::test_core::product2,
tests::test_core::product_temporary,
tests::test_core::izip_macro,
tests::test_core::izip2,
tests::test_core::izip3,
tests::test_core::multizip3,
tests::test_core::write_to,
tests::test_core::test_interleave,
tests::test_core::foreach,
tests::test_core::dropping,
tests::test_core::batching,
tests::test_core::test_put_back,
tests::test_core::step,
tests::test_core::merge,
tests::test_core::repeatn,
tests::test_core::count_clones,
tests::test_core::part,
tests::test_core::tree_fold1,
tests::test_core::exactly_one,
tests::test_std::product3,
tests::test_std::interleave_shortest,
tests::test_std::unique_by,
tests::test_std::unique,
tests::test_std::intersperse,
tests::test_std::dedup,
tests::test_std::all_equal,
tests::test_std::test_put_back_n,
tests::test_std::tee,
tests::test_std::test_rciter,
tests::test_std::trait_pointers,
tests::test_std::merge_by,
tests::test_std::merge_by_btree,
tests::test_std::kmerge,
tests::test_std::kmerge_2,
tests::test_std::kmerge_empty,
tests::test_std::kmerge_size_hint,
tests::test_std::kmerge_empty_size_hint,
tests::test_std::join,
tests::test_std::sorted_by,
tests::test_std::sorted_by_key,
tests::test_std::test_multipeek,
tests::test_std::test_multipeek_reset,
tests::test_std::test_multipeek_peeking_next,
tests::test_std::pad_using,
//tests::test_std::group_by,
tests::test_std::group_by_lazy_2,
tests::test_std::group_by_lazy_3,
tests::test_std::chunks,
tests::test_std::concat_empty,
tests::test_std::concat_non_empty,
tests::test_std::combinations,
tests::test_std::combinations_of_too_short,
tests::test_std::combinations_zero,
tests::test_std::diff_mismatch,
tests::test_std::diff_longer,
tests::test_std::diff_shorter,
tests::test_std::minmax,
tests::test_std::format,
tests::test_std::while_some,
tests::test_std::fold_while,
tests::test_std::tree_fold1,
tests::tuples::tuples,
tests::tuples::tuple_windows,
tests::tuples::next_tuple,
tests::tuples::collect_tuple,
tests::zip::zip_longest_fused,
tests::zip::test_zip_longest_size_hint,
tests::zip::test_double_ended_zip_longest,
|| should_panic!(tests::zip::zip_eq_panic1()),
|| should_panic!(tests::zip::zip_eq_panic2()),
);

    sgx_status_t::SGX_SUCCESS
}

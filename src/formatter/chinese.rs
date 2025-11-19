//! This module handles the conversion of numbers to various Chinese numeral formats.
//! It supports DBNum1, DBNum2, DBNum3, and DBNum4 as defined in Excel.

// DBNum1: Chinese Simplified Numerals (一, 二, 三)
const TRAD_SIMP_DIGITS: [&str; 10] = ["〇", "一", "二", "三", "四", "五", "六", "七", "八", "九"];
const TRAD_SIMP_UNITS: [&str; 4] = ["", "十", "百", "千"];

// DBNum2: Chinese Traditional Formal Numerals (壹, 贰, 叁)
const TRAD_FORMAL_DIGITS: [&str; 10] = ["零", "壹", "贰", "叁", "肆", "伍", "陆", "柒", "捌", "玖"];
const TRAD_FORMAL_UNITS: [&str; 4] = ["", "拾", "佰", "仟"];

// Shared units for large numbers
const LARGE_UNITS: [&str; 5] = ["", "万", "亿", "兆", "京"];

// DBNum4: Full-width digits
const FULL_WIDTH_DIGITS: [char; 10] = ['０', '１', '２', '３', '４', '５', '６', '７', '８', '９'];

/// Converts a number to a full-width string representation.
/// Corresponds to [DBNum4].
pub fn to_full_width(num: f64) -> String {
    num.to_string()
        .chars()
        .map(|c| match c {
            '0'..='9' => FULL_WIDTH_DIGITS[c.to_digit(10).unwrap() as usize],
            '.' => '．',
            '-' => '－',
            _ => c,
        })
        .collect()
}

/// Converts a number to a Chinese numeral string.
/// `is_formal` determines whether to use formal (DBNum2) or simplified (DBNum1) characters.
pub fn to_chinese_numeral(num: f64, is_formal: bool, use_leading_one_for_ten: bool) -> String {
    if !num.is_finite() || num.abs() > 9_999_999_999_999_999_999.0 {
        return num.to_string();
    }

    let (digits, units) = if is_formal {
        (TRAD_FORMAL_DIGITS, TRAD_FORMAL_UNITS)
    } else {
        (TRAD_SIMP_DIGITS, TRAD_SIMP_UNITS)
    };

    let mut result = String::new();

    if num.is_sign_negative() {
        result.push_str("负");
    }

    let num_abs = num.abs();
    let integer_part = num_abs.trunc() as u64;
    let fractional_part = num_abs.fract();

    if integer_part == 0 && fractional_part == 0.0 {
        result.push_str(digits[0]);
    } else if integer_part == 0 && fractional_part > 1e-9 {
        result.push_str(digits[0]);
    }
    else {
        result.push_str(&convert_integer(integer_part, &digits, &units, use_leading_one_for_ten));
    }

    if fractional_part > 1e-9 {
        result.push('.');
        let num_str = num.to_string();
        if let Some(dot_pos) = num_str.find('.') {
            let frac_part_str = &num_str[dot_pos + 1..];
            for ch in frac_part_str.chars() {
                if let Some(d) = ch.to_digit(10) {
                    result.push_str(digits[d as usize]);
                }
            }
        }
    }

    result
}

/// Converts a number to Chinese digits string.
/// Corresponds to [DBNum3].
pub fn to_chinese_digits(num: f64, is_formal: bool) -> String {
    let (digits, neg, point) = if is_formal {
        (
            TRAD_FORMAL_DIGITS,
            "负",
            ".",
        )
    } else {
        (
            TRAD_SIMP_DIGITS,
            "负",
            ".",
        )
    };

    let mut result = String::new();
    let s = num.to_string();

    for c in s.chars() {
        match c {
            '0'..='9' => result.push_str(digits[c.to_digit(10).unwrap() as usize]),
            '-' => result.push_str(neg),
            '.' => result.push_str(point),
            _ => {}
        }
    }
    result
}


// Helper to convert an integer part to Chinese numerals.
fn convert_integer(mut n: u64, digits: &[&str; 10], units: &[&str; 4], use_leading_one_for_ten: bool) -> String {
    if n == 0 {
        return digits[0].to_string();
    }

    let mut result = String::new();
    let mut unit_idx = 0;
    let mut needs_zero = false;

    while n > 0 {
        let part = n % 10000;
        if needs_zero {
            result.insert_str(0, digits[0]);
        }

        if part > 0 {
            let mut part_str = convert_four_digits(part, digits, units, use_leading_one_for_ten);
            if unit_idx > 0 {
                part_str.push_str(LARGE_UNITS[unit_idx]);
            }
            result.insert_str(0, &part_str);
            needs_zero = part < 1000 && n / 10000 > 0;
        } else {
            needs_zero = !result.is_empty() && !result.starts_with(digits[0]);
        }

        n /= 10000;
        unit_idx += 1;
    }

    result
}

fn convert_four_digits(mut n: u64, digits: &[&str; 10], units: &[&str; 4], use_leading_one_for_ten: bool) -> String {
    if n == 0 {
        return "".to_string();
    }

    let mut result = String::new();
    let mut was_zero = true;

    for i in (0..4).rev() {
        let base = 10u64.pow(i as u32);
        let d = n / base;

        if d > 0 {
            // Special case for "十" (shi). In "十二", it's "一十二".
            // In "二十", it's "二十".
            if d == 1 && i == 1 && n >= 10 && n < 20 {
                if use_leading_one_for_ten {
                    result.push_str(digits[1]);
                }
            } else {
                result.push_str(digits[d as usize]);
            }

            result.push_str(units[i]);
            was_zero = false;
        } else {
            if !was_zero && !result.is_empty() && !result.ends_with(digits[0]) {
                result.push_str(digits[0]);
            }
            was_zero = true;
        }
        n %= base;
    }

    // Remove trailing "〇"
    while result.ends_with(digits[0]) {
        result.pop();
    }

    result
}
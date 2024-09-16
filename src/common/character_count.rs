pub fn is_cjk(c: char) -> bool {
    // CJK文字のユニコード範囲を確認
    matches!(c,
        '\u{3040}'..='\u{309F}' |  // ひらがな
        '\u{30A0}'..='\u{30FF}' |  // カタカナ
        '\u{3400}'..='\u{4DBF}' |  // CJK統合漢字拡張A
        '\u{4E00}'..='\u{9FFF}' |  // CJK統合漢字
        '\u{F900}'..='\u{FAFF}' |  // CJK互換漢字
        '\u{FF00}'..='\u{FFEF}' |  // 全角・半角
        '\u{1100}'..='\u{11FF}' |  // ハングル字母
        '\u{AC00}'..='\u{D7AF}' |  // ハングル音節
        '\u{20000}'..='\u{2A6DF}' | // CJK統合漢字拡張B
        '\u{2A700}'..='\u{2B73F}' | // CJK統合漢字拡張C
        '\u{2B740}'..='\u{2B81F}' | // CJK統合漢字拡張D
        '\u{2B820}'..='\u{2CEAF}' | // CJK統合漢字拡張E
        '\u{2CEB0}'..='\u{2EBEF}'   // CJK統合漢字拡張F
    )
}

#[cfg(test)]
mod tests {
    use crate::common::character_count::is_cjk;

    // ある範囲の範囲外かつ他の範囲の範囲内の場合はコメントアウト

    // ひらがな (U+3040 - U+309F)
    #[test]
    fn test_hiragana_in_range() {
        assert!(is_cjk('\u{3040}'));
        assert!(is_cjk('\u{309F}'));
    }
    
    #[test]
    fn test_hiragana_out_of_range() {
        assert!(!is_cjk('\u{303F}'));  // 範囲外
        // assert!(!is_cjk('\u{30A0}'));  // 範囲直後
    }

    // カタカナ (U+30A0 - U+30FF)
    #[test]
    fn test_katakana_in_range() {
        assert!(is_cjk('\u{30A0}'));
        assert!(is_cjk('\u{30FF}'));
    }
    
    #[test]
    fn test_katakana_out_of_range() {
        // assert!(!is_cjk('\u{309F}'));  // 範囲外
        assert!(!is_cjk('\u{3100}'));  // 範囲直後
    }

    // CJK統合漢字拡張A (U+3400 - U+4DBF)
    #[test]
    fn test_cjk_extension_a_in_range() {
        assert!(is_cjk('\u{3400}'));
        assert!(is_cjk('\u{4DBF}'));
    }
    
    #[test]
    fn test_cjk_extension_a_out_of_range() {
        assert!(!is_cjk('\u{33FF}'));  // 範囲外
        assert!(!is_cjk('\u{4DC0}'));  // 範囲直後
    }

    // CJK統合漢字 (U+4E00 - U+9FFF)
    #[test]
    fn test_cjk_unified_ideographs_in_range() {
        assert!(is_cjk('\u{4E00}'));
        assert!(is_cjk('\u{9FFF}'));
    }

    #[test]
    fn test_cjk_unified_ideographs_out_of_range() {
        assert!(!is_cjk('\u{4DFF}'));  // 範囲外
        assert!(!is_cjk('\u{A000}'));  // 範囲直後
    }

    // CJK互換漢字 (U+F900 - U+FAFF)
    #[test]
    fn test_cjk_compatibility_in_range() {
        assert!(is_cjk('\u{F900}'));
        assert!(is_cjk('\u{FAFF}'));
    }
    
    #[test]
    fn test_cjk_compatibility_out_of_range() {
        assert!(!is_cjk('\u{F8FF}'));  // 範囲外
        assert!(!is_cjk('\u{FB00}'));  // 範囲直後
    }

    // 全角・半角 (U+FF00 - U+FFEF)
    #[test]
    fn test_fullwidth_halfwidth_in_range() {
        assert!(is_cjk('\u{FF00}'));
        assert!(is_cjk('\u{FFEF}'));
    }
    
    #[test]
    fn test_fullwidth_halfwidth_out_of_range() {
        assert!(!is_cjk('\u{FEFF}'));  // 範囲外
        assert!(!is_cjk('\u{FFF0}'));  // 範囲直後
    }

    // ハングル字母 (U+1100 - U+11FF)
    #[test]
    fn test_hangul_jamo_in_range() {
        assert!(is_cjk('\u{1100}'));
        assert!(is_cjk('\u{11FF}'));
    }
    
    #[test]
    fn test_hangul_jamo_out_of_range() {
        assert!(!is_cjk('\u{10FF}'));  // 範囲外
        assert!(!is_cjk('\u{1200}'));  // 範囲直後
    }

    // ハングル音節 (U+AC00 - U+D7AF)
    #[test]
    fn test_hangul_syllables_in_range() {
        assert!(is_cjk('\u{AC00}'));
        assert!(is_cjk('\u{D7AF}'));
    }
    
    #[test]
    fn test_hangul_syllables_out_of_range() {
        assert!(!is_cjk('\u{ABFF}'));  // 範囲外
        assert!(!is_cjk('\u{D7B0}'));  // 範囲直後
    }

    // CJK統合漢字拡張B (U+20000 - U+2A6DF)
    #[test]
    fn test_cjk_extension_b_in_range() {
        assert!(is_cjk('\u{20000}'));
        assert!(is_cjk('\u{2A6DF}'));
    }
    
    #[test]
    fn test_cjk_extension_b_out_of_range() {
        assert!(!is_cjk('\u{1FFFF}'));  // 範囲外
        assert!(!is_cjk('\u{2A6E0}'));  // 範囲直後
    }

    // CJK統合漢字拡張C (U+2A700 - U+2B73F)
    #[test]
    fn test_cjk_extension_c_in_range() {
        assert!(is_cjk('\u{2A700}'));
        assert!(is_cjk('\u{2B73F}'));
    }
    
    #[test]
    fn test_cjk_extension_c_out_of_range() {
        assert!(!is_cjk('\u{2A6FF}'));  // 範囲外
        // assert!(!is_cjk('\u{2B740}'));  // 範囲直後
    }

    // CJK統合漢字拡張D (U+2B740 - U+2B81F)
    #[test]
    fn test_cjk_extension_d_in_range() {
        assert!(is_cjk('\u{2B740}'));
        assert!(is_cjk('\u{2B81F}'));
    }
    
    #[test]
    fn test_cjk_extension_d_out_of_range() {
        // assert!(!is_cjk('\u{2B73F}'));  // 範囲外
        // assert!(!is_cjk('\u{2B820}'));  // 範囲直後
    }

    // CJK統合漢字拡張E (U+2B820 - U+2CEAF)
    #[test]
    fn test_cjk_extension_e_in_range() {
        assert!(is_cjk('\u{2B820}'));
        assert!(is_cjk('\u{2CEAF}'));
    }
    
    #[test]
    fn test_cjk_extension_e_out_of_range() {
        // assert!(!is_cjk('\u{2B81F}'));  // 範囲外
        // assert!(!is_cjk('\u{2CEB0}'));  // 範囲直後
    }

    // CJK統合漢字拡張F (U+2CEB0 - U+2EBEF)
    #[test]
    fn test_cjk_extension_f_in_range() {
        assert!(is_cjk('\u{2CEB0}'));
        assert!(is_cjk('\u{2EBEF}'));
    }
    
    #[test]
    fn test_cjk_extension_f_out_of_range() {
        // assert!(!is_cjk('\u{2CEA0}'));  // 範囲外
        assert!(!is_cjk('\u{2EBF0}'));  // 範囲直後
    }
}
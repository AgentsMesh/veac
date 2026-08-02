use super::*;

#[test]
fn input_policy_is_identical_for_string_and_os_commands() {
    let strings = string_arguments();
    let os_strings = os_arguments().map(|value| value.into_string().unwrap());
    assert_eq!(strings, os_strings);
    assert_eq!(strings[1], "file,pipe");
    for unsafe_format in ["hls", "concat", "dash", "imf", "image2"] {
        assert!(!strings[3].split(',').any(|value| value == unsafe_format));
    }
    for expected in ["mov", "matroska", "wav", "png_pipe"] {
        assert!(strings[3].split(',').any(|value| value == expected));
    }
}

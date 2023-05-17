use ellipse::Ellipse;

pub fn get_column_string(text: &str, width: usize) -> String {
    let elp = text.truncate_ellipse(width).to_string();
    if width == 0 {
        elp
    } else if width <= 3 {
        elp[3..].to_owned()
    } else if text.len() <= width {
        let padding_len = std::cmp::max(width - elp.len(), 0);
        let padding_raw = vec![b' '; padding_len];
        let padding = String::from_utf8_lossy(&padding_raw);
        format!("{}{}", &elp, &padding)
    } else {
        format!("{}{}", &elp[..width - 3], &elp[width..])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_column_string() {
        let text1 = "";
        let text2 = "test";
        let text3 = "testme";
        let text4 = "testmetest";

        let width = 0;

        assert_eq!(get_column_string(text4, width), "".to_owned());

        let width = 1;

        assert_eq!(get_column_string(text4, width), ".".to_owned());

        let width = 2;

        assert_eq!(get_column_string(text4, width), "..".to_owned());

        let width = 3;

        assert_eq!(get_column_string(text4, width), "...".to_owned());

        let width = 4;

        assert_eq!(get_column_string(text4, width), "t...".to_owned());

        let width = 6;

        assert_eq!(get_column_string(text1, width), "      ".to_owned());
        assert_eq!(get_column_string(text2, width), "test  ".to_owned());
        assert_eq!(get_column_string(text3, width), "testme".to_owned());
        assert_eq!(get_column_string(text4, width), "tes...".to_owned());
    }
}

pub fn logger_format() -> &'static str {
    return "🌐 %a \"%r\" %s %b \"%{Referer}i\" \"%{User-Agent}i\" %T";
}

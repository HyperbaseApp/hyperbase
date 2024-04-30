pub fn logger_format() -> &'static str {
    return "🌐 [ApiRestServer] %a \"%r\" %s %b \"%{Referer}i\" \"%{User-Agent}i\" %T";
}

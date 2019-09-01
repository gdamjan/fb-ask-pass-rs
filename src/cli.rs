pub struct Config {
    pub image_path: Option<String>,
    pub pass_path: Option<String>,
}

pub fn parse_args(args: &[String]) -> Result<Config, &'static str> {
    let str_args: Vec<&str> = args.iter().map(String::as_str).collect();

    let config = match str_args[..] {
        [_, "--image", img, "--write", path] | [_, "--write", path, "--image", img] => {
            Some(Config {
                image_path: Some(String::from(img)),
                pass_path: Some(String::from(path)),
            })
        }
        [_, "--write", path] => Some(Config {
            image_path: None,
            pass_path: Some(String::from(path)),
        }),
        [_, "--image", img] => Some(Config {
            image_path: Some(String::from(img)),
            pass_path: None,
        }),
        [_] => Some(Config {
            image_path: None,
            pass_path: None,
        }),
        _ => None,
    };
    config.ok_or("Possible arguments are --write and --image.")
}

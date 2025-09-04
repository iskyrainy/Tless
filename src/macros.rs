#[macro_export]
macro_rules! result_matcher {
    ($result:expr, $err_msg:expr) => {
        match $result {
            Err(err) => {
                eprintln!("{}: {}", $err_msg, err);
                std::process::exit(1);
            }
            Ok(_) => {}
        }
    };
    ($result:expr, $err_msg:expr, $suc_msg:expr) => {
        match $result {
            Err(err) => {
                eprintln!("{}: {}", $err_msg, err);
                std::process::exit(1);
            }
            Ok(_) => {
                println!("{}", $suc_msg);
            }
        }
    };
    ($result:expr, err_handler = $err_handler:expr) => {
        match $result {
            Err(err) => $err_handler(err),
            Ok(_) => {}
        }
    };
    ($result:expr, err_handler = $err_handler:expr, ok_handler = $ok_handler:expr) => {
        match $result {
            Err(err) => $err_handler(err),
            Ok(ok) => $ok_handler(ok),
        }
    };
}

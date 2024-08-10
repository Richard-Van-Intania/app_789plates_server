use chrono::{DateTime, Utc};
use sqlx::{Pool, Postgres};

pub async fn analyze_pattern(
    plates_id: i32,
    front_text: &String,
    front_number: i32,
    back_number: i32,
    add_date: DateTime<Utc>,
    vehicle_type_id: i32,
    pool: &Pool<Postgres>,
) {
    // constants
    // pattern_168
    if back_number == 168 {
        let _ = sqlx::query("INSERT INTO public.pattern_168(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_789
    if back_number == 789 {
        let _ = sqlx::query("INSERT INTO public.pattern_789(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_289
    if back_number == 289 {
        let _ = sqlx::query("INSERT INTO public.pattern_289(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_456
    if back_number == 456 {
        let _ = sqlx::query("INSERT INTO public.pattern_456(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_911
    if back_number == 911 {
        let _ = sqlx::query("INSERT INTO public.pattern_911(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_718
    if back_number == 718 {
        let _ = sqlx::query("INSERT INTO public.pattern_718(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_992
    if back_number == 992 {
        let _ = sqlx::query("INSERT INTO public.pattern_992(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_35
    if back_number == 35 {
        let _ = sqlx::query("INSERT INTO public.pattern_35(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_488
    if back_number == 488 {
        let _ = sqlx::query("INSERT INTO public.pattern_488(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_9
    if back_number == 9 {
        let _ = sqlx::query("INSERT INTO public.pattern_9(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_99
    if back_number == 99 {
        let _ = sqlx::query("INSERT INTO public.pattern_99(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_999
    if back_number == 999 {
        let _ = sqlx::query("INSERT INTO public.pattern_999(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_9999
    if back_number == 9999 {
        let _ = sqlx::query("INSERT INTO public.pattern_9999(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_7
    if back_number == 7 {
        let _ = sqlx::query("INSERT INTO public.pattern_7(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_77
    if back_number == 77 {
        let _ = sqlx::query("INSERT INTO public.pattern_77(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_777
    if back_number == 777 {
        let _ = sqlx::query("INSERT INTO public.pattern_777(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_7777
    if back_number == 7777 {
        let _ = sqlx::query("INSERT INTO public.pattern_7777(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_5
    if back_number == 5 {
        let _ = sqlx::query("INSERT INTO public.pattern_5(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_55
    if back_number == 55 {
        let _ = sqlx::query("INSERT INTO public.pattern_55(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_555
    if back_number == 555 {
        let _ = sqlx::query("INSERT INTO public.pattern_555(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_5555
    if back_number == 5555 {
        let _ = sqlx::query("INSERT INTO public.pattern_5555(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_8
    if back_number == 8 {
        let _ = sqlx::query("INSERT INTO public.pattern_8(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_88
    if back_number == 88 {
        let _ = sqlx::query("INSERT INTO public.pattern_88(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_888
    if back_number == 888 {
        let _ = sqlx::query("INSERT INTO public.pattern_888(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_8888
    if back_number == 8888 {
        let _ = sqlx::query("INSERT INTO public.pattern_8888(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_1
    if back_number == 1 {
        let _ = sqlx::query("INSERT INTO public.pattern_1(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_599
    if back_number == 599 {
        let _ = sqlx::query("INSERT INTO public.pattern_599(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_595
    if back_number == 595 {
        let _ = sqlx::query("INSERT INTO public.pattern_595(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_959
    if back_number == 959 {
        let _ = sqlx::query("INSERT INTO public.pattern_959(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_955
    if back_number == 955 {
        let _ = sqlx::query("INSERT INTO public.pattern_955(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_5959
    if back_number == 5959 {
        let _ = sqlx::query("INSERT INTO public.pattern_5959(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_9595
    if back_number == 9595 {
        let _ = sqlx::query("INSERT INTO public.pattern_9595(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_5599
    if back_number == 5599 {
        let _ = sqlx::query("INSERT INTO public.pattern_5599(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_9955
    if back_number == 9955 {
        let _ = sqlx::query("INSERT INTO public.pattern_9955(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_5995
    if back_number == 5995 {
        let _ = sqlx::query("INSERT INTO public.pattern_5995(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_9559
    if back_number == 9559 {
        let _ = sqlx::query("INSERT INTO public.pattern_9559(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // variable
    // pattern_x
    if back_number > 0 && back_number < 10 {
        let _ = sqlx::query("INSERT INTO public.pattern_x(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_xx
    if back_number == 11
        || back_number == 22
        || back_number == 33
        || back_number == 44
        || back_number == 55
        || back_number == 66
        || back_number == 77
        || back_number == 88
        || back_number == 99
    {
        let _ = sqlx::query("INSERT INTO public.pattern_xx(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_xxx
    if back_number == 111
        || back_number == 222
        || back_number == 333
        || back_number == 444
        || back_number == 555
        || back_number == 666
        || back_number == 777
        || back_number == 888
        || back_number == 999
    {
        let _ = sqlx::query("INSERT INTO public.pattern_xxx(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_xxxx
    if back_number == 1111
        || back_number == 2222
        || back_number == 3333
        || back_number == 4444
        || back_number == 5555
        || back_number == 6666
        || back_number == 7777
        || back_number == 8888
        || back_number == 9999
    {
        let _ = sqlx::query("INSERT INTO public.pattern_xxxx(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_xy
    if back_number > 9 && back_number < 100 {
        let list: Vec<char> = back_number.to_string().chars().collect();
        let a = list[0];
        let b = list[1];
        if a != b {
            let _ =
                sqlx::query("INSERT INTO public.pattern_xy(plates_id, add_date) VALUES ($1, $2)")
                    .bind(plates_id)
                    .bind(add_date)
                    .execute(pool)
                    .await;
        }
    }
    // pattern_xyy
    if back_number > 99 && back_number < 1000 {
        let list: Vec<char> = back_number.to_string().chars().collect();
        let a = list[0];
        let b = list[1];
        let c = list[2];
        if a != b && b == c {
            let _ =
                sqlx::query("INSERT INTO public.pattern_xyy(plates_id, add_date) VALUES ($1, $2)")
                    .bind(plates_id)
                    .bind(add_date)
                    .execute(pool)
                    .await;
        }
    }
    // pattern_xyyy
    if back_number > 999 && back_number < 10000 {
        let list: Vec<char> = back_number.to_string().chars().collect();
        let a = list[0];
        let b = list[1];
        let c = list[2];
        let d = list[3];
        if a != b && b == c && c == d {
            let _ =
                sqlx::query("INSERT INTO public.pattern_xyyy(plates_id, add_date) VALUES ($1, $2)")
                    .bind(plates_id)
                    .bind(add_date)
                    .execute(pool)
                    .await;
        }
    }
    // pattern_xxyy
    if back_number > 999 && back_number < 10000 {
        let list: Vec<char> = back_number.to_string().chars().collect();
        let a = list[0];
        let b = list[1];
        let c = list[2];
        let d = list[3];
        if a == b && c == d && a != c {
            let _ =
                sqlx::query("INSERT INTO public.pattern_xxyy(plates_id, add_date) VALUES ($1, $2)")
                    .bind(plates_id)
                    .bind(add_date)
                    .execute(pool)
                    .await;
        }
    }
    // pattern_xyxy
    if back_number > 999 && back_number < 10000 {
        let list: Vec<char> = back_number.to_string().chars().collect();
        let a = list[0];
        let b = list[1];
        let c = list[2];
        let d = list[3];
        if a == c && b == d && a != b {
            let _ =
                sqlx::query("INSERT INTO public.pattern_xyxy(plates_id, add_date) VALUES ($1, $2)")
                    .bind(plates_id)
                    .bind(add_date)
                    .execute(pool)
                    .await;
        }
    }
    // pattern_xyyx
    if back_number > 999 && back_number < 10000 {
        let list: Vec<char> = back_number.to_string().chars().collect();
        let a = list[0];
        let b = list[1];
        let c = list[2];
        let d = list[3];
        if a == d && b == c && a != b {
            let _ =
                sqlx::query("INSERT INTO public.pattern_xyyx(plates_id, add_date) VALUES ($1, $2)")
                    .bind(plates_id)
                    .bind(add_date)
                    .execute(pool)
                    .await;
        }
    }
    // pattern_xyx
    if back_number > 99 && back_number < 1000 {
        let list: Vec<char> = back_number.to_string().chars().collect();
        let a = list[0];
        let b = list[1];
        let c = list[2];
        if a != b && a == c {
            let _ =
                sqlx::query("INSERT INTO public.pattern_xyx(plates_id, add_date) VALUES ($1, $2)")
                    .bind(plates_id)
                    .bind(add_date)
                    .execute(pool)
                    .await;
        }
    }
    // pattern_xyz
    if back_number == 123
        || back_number == 234
        || back_number == 345
        || back_number == 456
        || back_number == 567
        || back_number == 678
        || back_number == 789
    {
        let _ = sqlx::query("INSERT INTO public.pattern_xyz(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_zyx
    if back_number == 987
        || back_number == 876
        || back_number == 765
        || back_number == 654
        || back_number == 543
        || back_number == 432
        || back_number == 321
    {
        let _ = sqlx::query("INSERT INTO public.pattern_zyx(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_wxyz
    if back_number == 1234
        || back_number == 2345
        || back_number == 3456
        || back_number == 4567
        || back_number == 5678
        || back_number == 6789
    {
        let _ = sqlx::query("INSERT INTO public.pattern_wxyz(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_zyxw
    if back_number == 9876
        || back_number == 8765
        || back_number == 7654
        || back_number == 6543
        || back_number == 5432
        || back_number == 4321
    {
        let _ = sqlx::query("INSERT INTO public.pattern_zyxw(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_x00
    if back_number == 100
        || back_number == 200
        || back_number == 300
        || back_number == 400
        || back_number == 500
        || back_number == 600
        || back_number == 700
        || back_number == 800
        || back_number == 900
    {
        let _ = sqlx::query("INSERT INTO public.pattern_x00(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_x000
    if back_number == 1000
        || back_number == 2000
        || back_number == 3000
        || back_number == 4000
        || back_number == 5000
        || back_number == 6000
        || back_number == 7000
        || back_number == 8000
        || back_number == 9000
    {
        let _ = sqlx::query("INSERT INTO public.pattern_x000(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_x99
    if back_number == 199
        || back_number == 299
        || back_number == 399
        || back_number == 499
        || back_number == 599
        || back_number == 699
        || back_number == 799
        || back_number == 899
        || back_number == 999
    {
        let _ = sqlx::query("INSERT INTO public.pattern_x99(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_x999
    if back_number == 1999
        || back_number == 2999
        || back_number == 3999
        || back_number == 4999
        || back_number == 5999
        || back_number == 6999
        || back_number == 7999
        || back_number == 8999
        || back_number == 9999
    {
        let _ = sqlx::query("INSERT INTO public.pattern_x999(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_x55
    if back_number == 155
        || back_number == 255
        || back_number == 355
        || back_number == 455
        || back_number == 555
        || back_number == 655
        || back_number == 755
        || back_number == 855
        || back_number == 955
    {
        let _ = sqlx::query("INSERT INTO public.pattern_x55(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_x555
    if back_number == 1555
        || back_number == 2555
        || back_number == 3555
        || back_number == 4555
        || back_number == 5555
        || back_number == 6555
        || back_number == 7555
        || back_number == 8555
        || back_number == 9555
    {
        let _ = sqlx::query("INSERT INTO public.pattern_x555(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_rakhang
    if front_number == 0 && vehicle_type_id == 1 && front_text.starts_with("ฆ") {
        let _ =
            sqlx::query("INSERT INTO public.pattern_rakhang(plates_id, add_date) VALUES ($1, $2)")
                .bind(plates_id)
                .bind(add_date)
                .execute(pool)
                .await;
    }
    // pattern_kob
    if front_text == "กบ" {
        let _ = sqlx::query("INSERT INTO public.pattern_kob(plates_id, add_date) VALUES ($1, $2)")
            .bind(plates_id)
            .bind(add_date)
            .execute(pool)
            .await;
    }
    // pattern_torthan
    if front_number == 0 && vehicle_type_id == 1 && front_text.starts_with("ฐ") {
        let _ =
            sqlx::query("INSERT INTO public.pattern_torthan(plates_id, add_date) VALUES ($1, $2)")
                .bind(plates_id)
                .bind(add_date)
                .execute(pool)
                .await;
    }
}

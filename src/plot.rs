use std::{collections::HashMap, error::Error, path::PathBuf, path::Path, fs, io::{self, Write}};

use chrono::{Utc, TimeZone, Months, Local};
use itertools::Itertools;
use plotters::{prelude::*, style::full_palette::GREEN_700};

pub(crate) fn plot<F>(
    weekly: HashMap<chrono::NaiveDate, u32>,
    daily: HashMap<chrono::NaiveDate, u32>,
    caption: &str,
    out: PathBuf,
    print_status: F
) -> Vec<PathBuf> 
where
    F: Fn()
{
    fs::create_dir_all(&out).unwrap();

    let weekly_file = out.join(&format!("{}_weekly.png", caption.to_lowercase().replace(" ", "_")));
    let daily_file = out.join(&format!("{}_daily.png", caption.to_lowercase().replace(" ", "_")));

    plot_stat(
        &format!("{} Daily", caption), 
        daily,
        daily_file.as_path())
        .unwrap_or_else(|err| {
            println!("Error plotting Daily: {}", err);
            ()
        });
    print_status();
    plot_stat(
        &format!("{} Weekly", caption), 
        weekly, 
        weekly_file.as_path())
        .unwrap_or_else(|err| {
            println!("Error plotting weekly: {}", err);
            ()
        });
    print_status();
    
    vec![daily_file, weekly_file]
}

fn plot_stat(
    caption: &str,
    data: HashMap<chrono::NaiveDate, u32>,
    out: &Path,
) -> Result<(), Box<dyn Error>> {
    let root = BitMapBackend::new(out, (1024, 768)).into_drawing_area();

    root.fill(&WHITE)?;

    let newest_date = Utc::now().date_naive();
    let oldest_date = data.keys().min().cloned().unwrap_or(newest_date);

    let three_months_ago = newest_date.checked_sub_months(Months::new(3)).unwrap();

    let max_progress = data.values().max().cloned().unwrap_or(10) + 2;

    let mut chart = ChartBuilder::on(&root)
        .margin(10)
        .margin_right(40)
        .caption(
            caption,
            ("sans-serif", 40)
        )
        .set_label_area_size(LabelAreaPosition::Left, 60)
        .set_label_area_size(LabelAreaPosition::Bottom, 40)
        .build_cartesian_2d(
            oldest_date.max(three_months_ago)..newest_date,
            0..max_progress,
        )?;

    chart
        .configure_mesh()
        .x_labels(if data.len() > 15 { data.len() / 2 } else { data.len() + 1 })
        .x_labels(30)
        .x_label_formatter(&|date| {
            let datetime = 
                Local
                .from_local_datetime(&date.and_hms_opt(0, 0, 0).unwrap())
                .unwrap();
            datetime.format("%m/%d").to_string()
        })
        .y_desc("Produced")
        .draw()?;

    chart.draw_series(LineSeries::new(
        data
                .iter()
                .filter(|(date, _)| {
                    date.cmp(&&three_months_ago).is_ge()
                })
                .sorted()
                .map(|(date, stat)| 
            (
                *date,
                *stat
            )),
            GREEN_700.stroke_width(3),
    ))?;

    chart.draw_series(
            data
                .iter()
                .filter(|(date, _)| {
                    date.cmp(&&three_months_ago).is_ge()
                })
                .sorted()
                .map(|(date, stat)| 
            Circle::new((
                *date,
                *stat
            ),
            5,
            GREEN_700.filled()))
    )?;

    root.present().expect("Unable to write result to file");
    print!("\r  saved to {}\n", out.to_str().unwrap());
    io::stdout().flush().unwrap();

    Ok(())
}
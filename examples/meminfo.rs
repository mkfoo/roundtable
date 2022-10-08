use plotters::prelude::*;
use roundtable as rt;
use rt::prelude::*;
use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::{Duration, SystemTime};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

rt::datapoint! {
    struct MemInfo {
        total: u32,
        free: u32,
        avail: u32,
        buffers: u32,
        cached: u32,
    }
}

fn record_data() -> rt::Result<InMemoryTable<MemInfo>> {
    let opts = Options::new(0, 5, 240);
    let mut table = rt::create::in_memory(opts, MemInfo::default())?;
    let now = SystemTime::now();

    for _ in 0..60 {
        sleep(Duration::new(5, 0));
        let meminfo = read_meminfo().unwrap_or_default();
        let t = now.elapsed().map(|d| d.as_secs()).unwrap_or_default();
        table.insert(t, &meminfo)?;
    }

    Ok(table)
}

fn read_meminfo() -> Option<MemInfo> {
    let out = Command::new("cat")
        .arg("/proc/meminfo")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap()
        .wait_with_output()
        .unwrap()
        .stdout;

    let mut meminfo = MemInfo::default();

    for line in String::from_utf8_lossy(&out).lines() {
        let mut i = line.split_whitespace();

        match (i.next()?, i.next()?.parse()) {
            ("MemTotal:", Ok(v)) => meminfo.total = v,
            ("MemFree:", Ok(v)) => meminfo.free = v,
            ("MemAvailable:", Ok(v)) => meminfo.avail = v,
            ("Buffers:", Ok(v)) => meminfo.buffers = v,
            ("Cached:", Ok(v)) => meminfo.cached = v,
            _ => break,
        }
    }

    Some(meminfo)
}

fn draw_chart(table: &mut InMemoryTable<MemInfo>) -> Result<()> {
    let start_time = table.first()?.0;
    let end_time = table.last()?.0;
    let mem_total = table.last()?.1.total;

    let root = SVGBackend::new("meminfo.svg", (1024, 768)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .x_label_area_size(30)
        .y_label_area_size(75)
        .margin(10)
        .caption("Meminfo", ("sans-serif", 30.0).into_font())
        .build_cartesian_2d(start_time..end_time, 0_u32..mem_total)?;

    chart.configure_mesh().x_desc("s").y_desc("KiB").draw()?;

    chart
        .configure_series_labels()
        .position(SeriesLabelPosition::UpperRight)
        .background_style(&RGBColor(160, 160, 160))
        .draw()?;

    let mut draw_line = |label, color| -> Result<()> {
        chart
            .draw_series(LineSeries::new(
                table.iter()?.map(|(t, v)| match label {
                    "MemFree" => (t, v.free),
                    "MemAvailable" => (t, v.avail),
                    "Buffers" => (t, v.buffers),
                    "Cached" => (t, v.cached),
                    _ => (t, v.total),
                }),
                color,
            ))?
            .label(label)
            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], color));
        Ok(())
    };

    draw_line("MemTotal", &RED)?;
    draw_line("MemFree", &BLUE)?;
    draw_line("MemAvailable", &GREEN)?;
    draw_line("Buffers", &YELLOW)?;
    draw_line("Cached", &MAGENTA)?;

    root.present().expect("drawing failed");
    Ok(())
}

fn main() {
    let mut table = record_data().unwrap();
    draw_chart(&mut table).unwrap();
}

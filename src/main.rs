use chrono::prelude::*;
use clap::Clap;
use std::io::Result;
use yahoo_finance_api as yahoo;

#[derive(Clap)]
#[clap(version = "1.0", author = "sdaigo")]

struct Opts {
    #[clap(short, long, default_value = "MSFT,GOOG,AAPL,UBER,IBM")]
    symbols: String,
    #[clap(short, long)]
    from: String,
}

///
/// Calculates the absolute / relative price change between the beginning and ending of f64 series.
///
/// # Returns
/// A tuple `(absolute, relative)` diff.
///
fn price_diff(a: &[f64]) -> Option<(f64, f64)> {
    if !a.is_empty() {
        let (first, last) = (a.first().unwrap(), a.last().unwrap());
        let diff = last - first;

        let first = if *first == 0.0 { 1.0 } else { *first };
        let rel_diff = diff / first;

        Some((diff, rel_diff))
    } else {
        None
    }
}

///
/// Calculate a simple moving average over the entire series.
///
fn n_window_sma(n: usize, series: &[f64]) -> Option<Vec<f64>> {
    if !series.is_empty() && n > 1 {
        Some(
            series
                .windows(n)
                .map(|w| w.iter().sum::<f64>() / w.len() as f64)
                .collect(),
        )
    } else {
        None
    }
}

///
/// Find the max value in a series of f64
///
fn max(series: &[f64]) -> Option<f64> {
    if series.is_empty() {
        None
    } else {
        Some(series.iter().fold(f64::MIN, |acc, q| acc.max(*q)))
    }
}

///
/// Find the min value in a series of f64
///
fn min(series: &[f64]) -> Option<f64> {
    if series.is_empty() {
        None
    } else {
        Some(series.iter().fold(f64::MAX, |acc, q| acc.min(*q)))
    }
}

fn main() -> Result<()> {
    let opts = Opts::parse();

    let provider = yahoo::YahooConnector::new();

    let symbols = opts.symbols.split(',');
    let from: DateTime<Utc> = opts.from.parse().expect("Failed to parse 'from' date");

    // print headers
    println!("period start,symbol,price,change %,min,max,30d avg");

    for symbol in symbols {
        if let Ok(response) = provider.get_quote_history(symbol, from, Utc::now()) {
            match response.quotes() {
                Ok(mut quotes) => {
                    if !quotes.is_empty() {
                        quotes.sort_by_cached_key(|k| k.timestamp);
                        let closes: Vec<f64> = quotes.iter().map(|q| q.adjclose as f64).collect();

                        if !closes.is_empty() {
                            let max_period: f64 = max(&closes).unwrap();
                            let min_period: f64 = min(&closes).unwrap();

                            let last_price = *closes.last().unwrap_or(&0.0);
                            let (_, pct_change) = price_diff(&closes).unwrap_or((0.0, 0.0));
                            let sma = n_window_sma(30, &closes).unwrap_or_default();

                            println!(
                                "{},{},{},{}%,${},${},${}",
                                from.to_rfc3339(),
                                symbol,
                                last_price,
                                pct_change * 100.0,
                                min_period,
                                max_period,
                                sma.last().unwrap_or(&0.0)
                            )
                        }
                    }
                }
                _ => {
                    eprint!("No quotes found '{}'", symbol);
                }
            }
        } else {
            eprint!("No quotes found '{}'", symbol);
        }
    }

    Ok(())
}

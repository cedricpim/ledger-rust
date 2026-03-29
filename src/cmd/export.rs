use clap::Parser;
use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::config::Config;
use crate::entity::date::Date;
use crate::entity::line::{Line, Liner};
use crate::entity::money::Currency;
use crate::exchange::Exchange;
use crate::resource::Resource;
use crate::Mode;

#[derive(Parser, Debug)]
pub struct Args {
    /// Directory where per-account CSV files will be written
    #[clap(short, long, default_value = ".")]
    output: String,
}

struct ExportRow {
    date: String,
    payee: String,
    notes: String,
    category: String,
    amount: String,
}

pub fn run(args: Args) -> anyhow::Result<()> {
    let config = Config::new()?;
    let exchange = Exchange::new(&config)?;
    let default_currency = Currency::parse(&config.currency)?;
    let mut resource = Resource::new(&config, Mode::Ledger)?;

    let mut exports: HashMap<String, Vec<ExportRow>> = HashMap::new();
    let mut marked: HashSet<usize> = HashSet::new();
    let mut pending_transfer: Option<(usize, Line)> = None;
    let mut index = 0usize;

    resource.line(&mut |record| {
        let current = index;
        index += 1;

        if record.category() == config.transfer {
            match pending_transfer.take() {
                None => {
                    if record.exported().is_empty() {
                        pending_transfer = Some((current, record.clone()));
                    }
                }
                Some((prev_idx, prev)) => {
                    if record.exported().is_empty() {
                        let (source, source_idx, dest) = if prev.amount().negative() {
                            (prev, prev_idx, record.clone())
                        } else {
                            (record.clone(), current, prev)
                        };
                        let dest_idx =
                            if source_idx == prev_idx { current } else { prev_idx };

                        exports
                            .entry(source.account())
                            .or_default()
                            .push(ExportRow {
                                date: source.date().to_string(),
                                payee: dest.account(),
                                notes: build_notes(&source),
                                category: source.category(),
                                amount: format_amount(&source, default_currency, &exchange)?,
                            });

                        marked.insert(source_idx);
                        marked.insert(dest_idx);
                    }
                }
            }
        } else {
            pending_transfer = None;

            if record.exported().is_empty() {
                exports
                    .entry(record.account())
                    .or_default()
                    .push(ExportRow {
                        date: record.date().to_string(),
                        payee: record.venue(),
                        notes: build_notes(record),
                        category: record.category(),
                        amount: format_amount(record, default_currency, &exchange)?,
                    });

                marked.insert(current);
            }
        }

        Ok(())
    })?;

    let output_dir = Path::new(&args.output);

    for (account, rows) in &exports {
        let path = output_dir.join(format!("{}.csv", sanitize(account)));
        let mut wrt = csv::Writer::from_path(&path)?;
        wrt.write_record(["Date", "Payee", "Notes", "Category", "Amount"])?;
        for row in rows {
            wrt.write_record([
                &row.date, &row.payee, &row.notes, &row.category, &row.amount,
            ])?;
        }
        wrt.flush()?;
    }

    let today = Date::today().to_string();
    let mut rewrite_index = 0usize;

    resource.rewrite(&mut |record| {
        let current = rewrite_index;
        rewrite_index += 1;

        if marked.contains(&current) {
            record.set_exported(today.clone());
        }

        Ok(vec![record.clone()])
    })?;

    // Networth entries
    let mut nw_resource = Resource::new(&config, Mode::Networth)?;
    let mut nw_exports: HashMap<String, Vec<ExportRow>> = HashMap::new();
    let mut nw_marked: HashSet<usize> = HashSet::new();
    let mut nw_index = 0usize;
    let mut previous_investment: Option<crate::entity::money::Money> = None;

    nw_resource.line(&mut |record| {
        let current = nw_index;
        nw_index += 1;

        let prev = previous_investment;
        previous_investment = Some(record.investment());

        if record.exported().is_empty() {
            let zero = crate::entity::money::Money::new(record.currency(), 0);
            let delta = record.investment() - prev.unwrap_or(zero);
            let amount = delta.exchange(default_currency, &exchange)?;
            let prec = default_currency.decimal_places() as usize;

            nw_exports
                .entry(record.account())
                .or_default()
                .push(ExportRow {
                    date: record.date().to_string(),
                    payee: String::new(),
                    notes: "Daily".to_string(),
                    category: record.category(),
                    amount: format!("{:.prec$}", amount.to_number()),
                });

            nw_marked.insert(current);
        }

        Ok(())
    })?;

    for (account, rows) in &nw_exports {
        let path = output_dir.join(format!("{}.csv", sanitize(account)));
        let mut wrt = csv::Writer::from_path(&path)?;
        wrt.write_record(["Date", "Payee", "Notes", "Category", "Amount"])?;
        for row in rows {
            wrt.write_record([
                &row.date, &row.payee, &row.notes, &row.category, &row.amount,
            ])?;
        }
        wrt.flush()?;
    }

    if !nw_marked.is_empty() {
        let mut nw_rewrite_index = 0usize;
        nw_resource.rewrite(&mut |record| {
            let current = nw_rewrite_index;
            nw_rewrite_index += 1;
            if nw_marked.contains(&current) {
                record.set_exported(today.clone());
            }
            Ok(vec![record.clone()])
        })?;
    }

    let total: usize = exports.values().map(|v| v.len()).sum::<usize>()
        + nw_exports.values().map(|v| v.len()).sum::<usize>();

    if total == 0 {
        println!("Nothing to export.");
    } else {
        let accounts = exports.len() + nw_exports.len();
        println!(
            "Exported {} transaction(s) across {} account(s).",
            total, accounts
        );
    }

    Ok(())
}

fn build_notes(line: &Line) -> String {
    let desc = line.description();
    let qty = line.quantity();
    if qty.is_empty() {
        desc
    } else {
        format!("{} [{}]", desc, qty)
    }
}

fn format_amount(line: &Line, currency: Currency, exchange: &Exchange) -> anyhow::Result<String> {
    let money = line.amount().exchange(currency, exchange)?;
    let prec = currency.decimal_places() as usize;
    Ok(format!("{:.prec$}", money.to_number()))
}

fn sanitize(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' { c } else { '_' })
        .collect()
}

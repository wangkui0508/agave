#![allow(clippy::arithmetic_side_effects)]

#[macro_use]
extern crate log;
use {
    clap::{crate_description, crate_name, value_t, App, Arg},
    rayon::prelude::*,
    solana_accounts_db::{
        accounts::Accounts,
        accounts_db::{
            test_utils::{create_test_accounts_TT, update_accounts_bench_TT},
            AccountShrinkThreshold, AccountsDb, CalcAccountsHashDataSource,
            ACCOUNTS_DB_CONFIG_FOR_BENCHMARKS,
        },
        accounts_index::AccountSecondaryIndexes,
        ancestors::Ancestors,
    },
    solana_measure::measure::Measure,
    solana_sdk::{
        genesis_config::ClusterType, pubkey::Pubkey, rent_collector::RentCollector,
        sysvar::epoch_schedule::EpochSchedule,
    },
    std::{env, fs, path::PathBuf, sync::Arc},
};
use std::time::Instant;

fn main() {
    solana_logger::setup();

    let matches = App::new(crate_name!())
        .about(crate_description!())
        .version(solana_version::version!())
        .arg(
            Arg::with_name("num_slots")
                .long("num_slots")
                .takes_value(true)
                .value_name("SLOTS")
                .help("Number of slots to store to."),
        )
        .arg(
            Arg::with_name("num_accounts")
                .long("num_accounts")
                .takes_value(true)
                .value_name("NUM_ACCOUNTS")
                .help("Total number of accounts"),
        )
        .get_matches();

    let num_slots = value_t!(matches, "num_slots", usize).unwrap_or(4);
    let num_accounts = value_t!(matches, "num_accounts", usize).unwrap_or(10_000);
    let iterations = value_t!(matches, "iterations", usize).unwrap_or(20);
    let clean = matches.is_present("clean");
    println!("clean: {clean:?}");

    let path = PathBuf::from(env::var("FARF_DIR").unwrap_or_else(|_| "farf".to_owned()))
        .join("accounts-bench");
    println!("cleaning file system: {path:?}");
    if fs::remove_dir_all(path.clone()).is_err() {
        println!("Warning: Couldn't remove {path:?}");
    }
    let accounts_db = AccountsDb::new_with_config(
        vec![path],
        &ClusterType::Testnet,
        AccountSecondaryIndexes::default(),
        AccountShrinkThreshold::default(),
        Some(ACCOUNTS_DB_CONFIG_FOR_BENCHMARKS),
        None,
        Arc::default(),
    );
    let accounts = Accounts::new(Arc::new(accounts_db));
    println!("Creating {num_accounts} accounts");
    let mut create_time = Measure::start("create accounts");
    let acc_in_slot = num_accounts / num_slots;
    let mut start = Instant::now();
    for j in 0..(num_slots / 20) {
        println!("Now {} elpased {:.2?}", j * 20, start.elapsed());
        let range = (j * 20..j * 20 + 20);
        std::thread::scope(|s| {
            for slot in range.clone() {
                let accounts = &accounts;
                s.spawn(move|| {
                    create_test_accounts_TT(
                        accounts,
                        acc_in_slot,
                        slot as u64,
                    );
                });
            }
        });
        for i in range {
            accounts.add_root(i as u64);
        }
    }
    create_time.stop();
    println!(
        "created {} accounts in {} slots {}",
        (num_accounts / num_slots) * num_slots,
        num_slots,
        create_time
    );

    let mut time = Measure::start("update");
    let num_update_slots = num_slots / 5;
    for j in 0..num_update_slots / 20 {
        let start = num_slots + j * 20;
        let range = (start..start + 20);
        std::thread::scope(|s| {
            for slot in range.clone() {
                let accounts = &accounts;
                s.spawn(move|| {
                    update_accounts_bench_TT(accounts, acc_in_slot, num_accounts, slot as u64);
                });
            }
        });
        for i in range {
            accounts.add_root(i as u64);
        }
    }
    time.stop();
    println!("update time {}", time);
}

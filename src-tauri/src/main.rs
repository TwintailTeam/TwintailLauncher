// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![feature(async_closure)]

fn main() {
    keqinglauncher_lib::run()
}

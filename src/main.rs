fn main() -> anyhow::Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_hal::task::critical_section::link();
    esp_idf_svc::timer::embassy_time::driver::link();
    esp_idf_svc::timer::embassy_time::queue::link();
    esp_idf_svc::log::EspLogger::initialize_default();

    let executor = esp_idf_hal::task::executor::EspExecutor::<4, _>::new();
    let task = executor.spawn_local(async {
        let mut counter = 0;
        loop {
            log::info!("counter at {counter}");
            embassy_time::Timer::after(embassy_time::Duration::from_millis(10)).await;
            counter += 1;
        }
    })?;

    executor.run_tasks(|| true, [task]);

    Ok(())
}

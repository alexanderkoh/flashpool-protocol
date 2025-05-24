use soroban_sdk::Address;

pub fn log_pair_creation(
    pair_addr: &Address,
    token0_label: &str,
    token0_addr: &Address,
    token1_label: &str,
    token1_addr: &Address,
) {
    std::println!("     PAIR CREATED\n          {:?}", pair_addr);
    std::println!(
        "             token0: {t0}\n             Address:    {:?}",
        token0_addr,
        t0 = token0_label
    );
    std::println!(
        "             token1: {t1}\n             Address:    {:?}",
        token1_addr,
        t1 = token1_label
    );
}
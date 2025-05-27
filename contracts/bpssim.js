// Flash Tokenomics Simulator - Core Math

// Constants and helper functions
const UNIT = 1e7;
const rnd = (a, b) => a + Math.random() * (b - a);

// instead of the binary search let's try this. we will scale bps based on the circ supply.
function findOptimalSwapWithSkew(
    campaign_fee_USDC,
    LP_USDC_reserves_start,
    LP_FLASH_reserves_start,
    swapOut,
    surplus_bps = 500, // e.g. 500 = 5%
    max_bps = 10000
) {
    // Minimum swap to keep pool ratio
    let s_min = Math.round(
        Math.sqrt(LP_USDC_reserves_start * (LP_USDC_reserves_start + campaign_fee_USDC)) - LP_USDC_reserves_start
    );
    // Surplus swap (extra emission)
    let swap_usd1 = Math.min(
        s_min + Math.floor(campaign_fee_USDC * surplus_bps / max_bps),
        campaign_fee_USDC
    );
    let flash_out = swapOut(swap_usd1, LP_USDC_reserves_start, LP_FLASH_reserves_start);
    let USD_Liquidity_Estimate = campaign_fee_USDC - swap_usd1;
    let flash_needed = Math.ceil(
        USD_Liquidity_Estimate * (LP_FLASH_reserves_start - flash_out) / (LP_USDC_reserves_start + swap_usd1)
    );
    return {
        swap_usd: swap_usd1,
        flash_out,
        flash_needed,
        iterations: 1
    };
}

function biasedDeposit(i, total, oMin, oMax, mMin, mMax) {
    const base = Math.random() < 0.99 ? rnd(mMin, mMax) : rnd(oMin, oMax);
    const t = (i - 1) / (total - 1);
    const scale = 1 + Math.pow(t, 2.5);
    return round(Math.min(base * scale, oMax));
}

// Default parameters
const Config = {
    total: 10_000_000,     // Total supply (FLASH)
    initF: 250_000,       // Initial LP FLASH
    initU: 250,          // Initial LP USDC
    oMin: 25,            // Outer min deposit
    oMax: 2500,          // Outer max deposit
    mMin: 10,            // 99% between min
    mMax: 500,           // 99% between max
    auto: true,          // Use analytic ideal swap
    rewardPct: 0.8,     // Treasury reward % of liqF
    sellPct: 80,          // Sell-back % of rewards
    campaigns: 25000,      // Number of campaigns
    disp: 100,            // Display every N rows
    MAX_BPS: 10000, // Max basis points (100% = 10_000)
};
function swapOut(aIn, rIn, rOut) {
    const fee = Math.ceil(aIn * .003);            // 0.3 %
    const net = aIn - fee;                        // net after fee
    return round(Math.floor(net * rOut / (rIn + net)));  // ΔY = Δx·Y/(X+Δx)
}

function round(num) {
    return Number(num.toFixed(0));
}
// Core simulation function
function simulate(params = {}) {
    let iteration_counts = [];

    const MAX = Config.total * UNIT;
    let lpF = Config.initF * UNIT;
    let lpU = Config.initU * UNIT;
    let total_emissions = 0;
    let treasury_flash = (MAX - lpF);
    let lp_reserve_flash = lpF;
    let lp_reserve_usdc = lpU;
    
    let prev_reward_emitted = 0;
    const results = [];
    ProtocolState = { ...Config }
    for (let campaign_number = 1; campaign_number <= Config.campaigns; campaign_number++) {
        // the amount of usdc the creator deposits to back their campaign.
        const campaign_fee_USDC = campaign_number === 1 ? 100 * UNIT : biasedDeposit(campaign_number, Config.campaigns, Config.oMin, Config.oMax, Config.mMin, Config.mMax) * UNIT;
        const LP_FLASH_reserves_start = lp_reserve_flash;
        const LP_USDC_reserves_start = lp_reserve_usdc;

        const circulating_pct = (MAX - treasury_flash) / MAX; 
        //not sure if this should be a linear function or exponential..in the contracts it can be configurable i guess.. 
        const target_max_surplus = Math.pow(circulating_pct, 1.2); //(lp_reserve_usdc/lp_reserve_flash);
        const MAX_BPS = Config.MAX_BPS;
        const surplus_bps = Math.round(MAX_BPS * circulating_pct * target_max_surplus);
        const { swap_usd, flash_out, flash_needed, iterations } = findOptimalSwapWithSkew( // findOptimalSwapUSD(
            campaign_fee_USDC,
            lp_reserve_usdc,
            lp_reserve_flash,
            swapOut,
            surplus_bps
        );

        const flash_out_check = swapOut(swap_usd, LP_USDC_reserves_start, LP_FLASH_reserves_start);
        if (flash_out !== flash_out_check) {
            console.error('Flash out mismatch:', flash_out, flash_out_check);
        }

        const start_TVL = round(LP_USDC_reserves_start * 2);

        lp_reserve_flash -= flash_out;
        lp_reserve_usdc += swap_usd;

        treasury_flash += round(flash_out);

        const LP_FLASH_reserves_after_swap = round(LP_FLASH_reserves_start - flash_out);
        const LP_USDC_reserves_after_swap = round(LP_USDC_reserves_start + swap_usd);

        const USD_liquidity = campaign_fee_USDC - swap_usd

        const LP_USDC_reserves_after_deposit = LP_USDC_reserves_after_swap + USD_liquidity;
        const LP_FLASH_reserves_after_deposit = LP_FLASH_reserves_after_swap + flash_needed;
        lp_reserve_flash = LP_FLASH_reserves_after_deposit;
        lp_reserve_usdc = LP_USDC_reserves_after_deposit;
        treasury_flash -= flash_needed;
        // Determine flash available for rewards based on treasury and max sell limits
        let excess_flash = flash_out - flash_needed;
        const tvl_ratio = 2 * LP_USDC_reserves_after_deposit / start_TVL;
        const max_flash_to_sell = tvl_ratio > 1 ?
            round(lp_reserve_flash * (tvl_ratio - 1) * 0.999) : 0; // 0.999 as safety buffer
       // console.log(max_flash_to_sell, tvl_ratio, lp_reserve_flash, LP_USDC_reserves_after_deposit, start_TVL);
        const flash_for_rewards_from_swap = excess_flash > 0 ? excess_flash : 0 //Math.min(excess_flash, max_flash_to_sell) : 0;
        treasury_flash -= flash_for_rewards_from_swap;

        let flash_for_rewards_from_treasury = round(Math.min(
            Math.max(0, flash_out - flash_for_rewards_from_swap),
            max_flash_to_sell
        ) * Config.rewardPct);

        if (flash_for_rewards_from_treasury > treasury_flash) {
            flash_for_rewards_from_treasury = treasury_flash;
        }
        //const total_rewards_for_campaign1 = swap_emit + treasury_emit;

        const total_rewards_for_campaign = flash_for_rewards_from_treasury + flash_for_rewards_from_swap;
        //treasury_flash = Treasury_Start_Bal - treasury_emit;

        //treasury_flash = Treasury_Start_Bal - flash_for_rewards_from_treasury;
        treasury_flash -= flash_for_rewards_from_treasury;
        const updated_circulating_supply = MAX - treasury_flash;
        const maximum_to_sell_from_last_campaign = prev_reward_emitted

        const flash_sold_this_campaign = maximum_to_sell_from_last_campaign * (Config.sellPct / 100);

        const output_usd_from_flash_swaps = flash_sold_this_campaign > 0 ? swapOut(flash_sold_this_campaign, LP_FLASH_reserves_after_deposit, LP_USDC_reserves_after_deposit) : 0;

        const LP_USDC_reserves_after_sellback = LP_USDC_reserves_after_deposit - output_usd_from_flash_swaps;
        const LP_FLASH_reserves_after_sellback = LP_FLASH_reserves_after_deposit + flash_sold_this_campaign;

        lp_reserve_usdc -= output_usd_from_flash_swaps;
        lp_reserve_flash += flash_sold_this_campaign;

        const TVL_after_sellback = LP_USDC_reserves_after_sellback * 2;

        const final_price = LP_USDC_reserves_after_sellback / LP_FLASH_reserves_after_sellback;
        prev_reward_emitted = round(total_rewards_for_campaign);
        total_emissions += prev_reward_emitted;

        const circulating_percentage_of_total_supply = updated_circulating_supply / (Config.total * UNIT);

        // price and supply numbers again
        const price = LP_USDC_reserves_after_sellback / LP_FLASH_reserves_after_sellback

        if (campaign_number % Config.disp === 0 || campaign_number === 1 || campaign_number === Config.campaigns) {
            results.push({
                USD_FEE: (campaign_fee_USDC / UNIT).toFixed(2),
                Swap_U_In: (swap_usd / UNIT).toFixed(2),
                SwapFout: (flash_out / UNIT).toFixed(2),
                //USD_Liq: (USD_liquidity / UNIT).toFixed(2),
                Flsh_Liq: (flash_needed / UNIT).toFixed(2),
                max_emit: (max_flash_to_sell / UNIT).toFixed(2),
                mgrEmit: (flash_for_rewards_from_treasury / UNIT).toFixed(2),
                swapEmit: (flash_for_rewards_from_swap / UNIT).toFixed(2),
                surbps: surplus_bps,
                totEmit: (total_rewards_for_campaign / UNIT).toFixed(2),
                pctMax: (total_rewards_for_campaign / max_flash_to_sell).toFixed(2),
                treasury: (treasury_flash / UNIT).toFixed(0),
                lp_r_f: (LP_FLASH_reserves_after_sellback / UNIT).toFixed(0),
                lp_r_u: (LP_USDC_reserves_after_sellback / UNIT).toFixed(0),
                price: price.toFixed(4),
                //TVL: (TVL_after_sellback / UNIT).toFixed(2),
                circ: (updated_circulating_supply / UNIT).toFixed(0),
                circPct: circulating_percentage_of_total_supply.toFixed(2),
            });
        }

        iteration_counts.push(iterations);

    }

    // Print final results
    const final = results[results.length - 1];


    return { results, iteration_counts };
}



// Usage example
const { results, iteration_counts } = simulate();
const avg_iterations = iteration_counts.reduce((a, b) => a + b, 0) / iteration_counts.length;

console.table(results);
console.log(`\nAverage binary search iterations per campaign: ${avg_iterations.toFixed(2)}`);

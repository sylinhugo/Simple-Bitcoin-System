// SPDX-License-Identifier: MIT
pragma solidity >=0.8.0 <0.9.0;

import "@chainlink/contracts/src/v0.8/interfaces/AggregatorV3Interface.sol";
import "./interfaces/IPriceFeed.sol";

contract PriceFeed is IPriceFeed {
    /* TODO: implement your functions here */

    AggregatorV3Interface internal priceFeed;

    /**
     * Network: Kovan
     * Aggregator: ETH/USD
     * Address: 0x9326BFA02ADD2366b30bacB125260Af641031331
     */
    constructor() {
        priceFeed = AggregatorV3Interface(
            0xb31357d152638fd1ae0853d24b9Ea81dF29E3EF2
        );
    }

    /**
     * Returns the latest price
     */
    function getLatestPrice() public view override returns (int256, uint256) {
        (
            ,
            /*uint80 roundID*/
            int256 price,
            ,
            /*uint startedAt*/
            uint256 timeStamp,

        ) = /*uint80 answeredInRound*/
            priceFeed.latestRoundData();
        return (price, timeStamp);
    }
}

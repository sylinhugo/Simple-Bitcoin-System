// SPDX-License-Identifier: MIT
pragma solidity >=0.8.0 <0.9.0;
import "@openzeppelin/contracts/access/Ownable.sol";
import "./interfaces/ISwap.sol";
import "./sAsset.sol";

contract Swap is Ownable, ISwap {
    address token0;
    address token1;
    uint256 reserve0;
    uint256 reserve1;
    mapping(address => uint256) shares;
    uint256 public totalShares;

    constructor(address addr0, address addr1) {
        token0 = addr0;
        token1 = addr1;
    }

    function init(uint256 token0Amount, uint256 token1Amount)
        external
        override
        onlyOwner
    {
        require(reserve0 == 0 && reserve1 == 0, "init - already has liquidity");
        require(
            token0Amount > 0 && token1Amount > 0,
            "init - both tokens are needed"
        );

        require(
            sAsset(token0).transferFrom(msg.sender, address(this), token0Amount)
        );
        require(
            sAsset(token1).transferFrom(msg.sender, address(this), token1Amount)
        );
        reserve0 = token0Amount;
        reserve1 = token1Amount;
        totalShares = sqrt(token0Amount * token1Amount);
        shares[msg.sender] = totalShares;
    }

    // https://github.com/Uniswap/v2-core/blob/v1.0.1/contracts/libraries/Math.sol
    function sqrt(uint256 y) internal pure returns (uint256 z) {
        if (y > 3) {
            z = y;
            uint256 x = y / 2 + 1;
            while (x < z) {
                z = x;
                x = (y / x + x) / 2;
            }
        } else if (y != 0) {
            z = 1;
        }
    }

    function getReserves() external view returns (uint256, uint256) {
        return (reserve0, reserve1);
    }

    function getTokens() external view returns (address, address) {
        return (token0, token1);
    }

    function getShares(address LP) external view returns (uint256) {
        return shares[LP];
    }

    /* TODO: implement your functions here */

    function addLiquidity(uint256 token0Amount) external override {}

    function removeLiquidity(uint256 withdrawShares) external override {}

    function token0To1(uint256 token0Amount) external override {}

    function token1To0(uint256 token1Amount) external override {}
}

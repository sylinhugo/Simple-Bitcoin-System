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

    function addLiquidity(uint256 token0Amount) external override {
        (uint256 _reserve0, uint256 _reserve1) = (reserve0, reserve1);
        (address _token0, address _token1) = (token0, token1);
        uint256 amount0;
        uint256 amount1;
        uint256 newShares;

        amount0 = token0Amount;
        amount1 = (_reserve1 * amount0) / _reserve0;
        newShares = (totalShares * amount0) / _reserve0;

        require(
            sAsset(_token0).transferFrom(msg.sender, address(this), amount0)
        );
        // require(sAsset(_token0).mint(msg.sender, amount0));
        require(
            sAsset(_token1).transferFrom(msg.sender, address(this), amount1)
        );
        // require(sAsset(_token1).mint(msg.sender, amount1));

        totalShares += newShares;
        (_reserve0, _reserve1) = (_reserve0 + amount0, _reserve1 + amount1);
        (reserve0, reserve1) = (_reserve0, _reserve1);
        shares[msg.sender] += newShares;
    }

    function removeLiquidity(uint256 withdrawShares) external override {
        (uint256 _reserve0, uint256 _reserve1) = (reserve0, reserve1);
        (address _token0, address _token1) = (token0, token1);
        uint256 amount0;
        uint256 amount1;

        require(
            shares[msg.sender] >= withdrawShares,
            "shares[msg.sender] don't have enough shares"
        );

        amount0 = (_reserve0 * withdrawShares) / totalShares;
        amount1 = (_reserve1 * withdrawShares) / totalShares;

        require(
            sAsset(_token0).balanceOf(address(this)) >= amount0,
            "_token0 don't have enough balance"
        );
        require(
            sAsset(_token1).balanceOf(address(this)) >= amount1,
            "_token1 don't have enough balance"
        );

        require(sAsset(_token0).transfer(msg.sender, amount0));
        require(sAsset(_token1).transfer(msg.sender, amount1));

        // require(sAsset(_token0).burn(msg.sender, amount0));
        // require(sAsset(_token1).burn(msg.sender, amount1));

        (_reserve0, _reserve1) = (_reserve0 - amount0, _reserve1 - amount1);
        (reserve0, reserve1) = (_reserve0, _reserve1);
        // uint256 newShares = (totalShares * amount0) / reserve0;
        shares[msg.sender] -= withdrawShares;
    }

    function token0To1(uint256 token0Amount) external override {
        (uint256 _reserve0, uint256 _reserve1) = (reserve0, reserve1);
        (address _token0, address _token1) = (token0, token1);

        uint256 token0_sent = token0Amount;
        uint256 token0_to_exchange = (token0_sent * 997) / 1000;
        uint256 token1_to_return = _reserve1 -
            (_reserve0 * _reserve1) /
            (_reserve0 + token0_to_exchange);

        require(
            sAsset(_token0).transferFrom(msg.sender, address(this), token0_sent)
        );
        require(sAsset(_token1).transfer(msg.sender, token1_to_return));

        reserve0 += token0_sent;
        reserve1 -= token1_to_return;
    }

    function token1To0(uint256 token1Amount) external override {
        (uint256 _reserve0, uint256 _reserve1) = (reserve0, reserve1);
        (address _token0, address _token1) = (token0, token1);

        uint256 token1_sent = token1Amount;
        uint256 token1_to_exchange = (token1_sent * 997) / 1000;
        uint256 token0_to_return = _reserve0 -
            (_reserve1 * _reserve0) /
            (_reserve1 + token1_to_exchange);

        require(
            sAsset(_token1).transferFrom(msg.sender, address(this), token1_sent)
        );
        require(sAsset(_token0).transfer(msg.sender, token0_to_return));

        reserve1 += token1Amount;
        reserve0 -= token0_to_return;
    }
}

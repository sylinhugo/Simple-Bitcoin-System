// SPDX-License-Identifier: MIT
pragma solidity >=0.8.0 <0.9.0;

import "@openzeppelin/contracts/access/Ownable.sol";
import "./interfaces/IPriceFeed.sol";
import "./interfaces/IMint.sol";
import "./sAsset.sol";
import "./EUSD.sol";

contract Mint is Ownable, IMint {
    struct Asset {
        address token;
        uint256 minCollateralRatio;
        address priceFeed;
    }

    struct Position {
        uint256 idx;
        address owner;
        uint256 collateralAmount;
        address assetToken;
        uint256 assetAmount;
    }

    mapping(address => Asset) _assetMap;
    uint256 _currentPositionIndex;
    mapping(uint256 => Position) _idxPositionMap;
    address public collateralToken;

    constructor(address collateral) {
        collateralToken = collateral;
    }

    function registerAsset(
        address assetToken,
        uint256 minCollateralRatio,
        address priceFeed
    ) external override onlyOwner {
        require(assetToken != address(0), "Invalid assetToken address");
        require(
            minCollateralRatio >= 1,
            "minCollateralRatio must be greater than 100%"
        );
        require(
            _assetMap[assetToken].token == address(0),
            "Asset was already registered"
        );

        _assetMap[assetToken] = Asset(
            assetToken,
            minCollateralRatio,
            priceFeed
        );
    }

    function getPosition(uint256 positionIndex)
        external
        view
        returns (
            address,
            uint256,
            address,
            uint256
        )
    {
        require(positionIndex < _currentPositionIndex, "Invalid index");
        Position storage position = _idxPositionMap[positionIndex];
        return (
            position.owner,
            position.collateralAmount,
            position.assetToken,
            position.assetAmount
        );
    }

    function getMintAmount(
        uint256 collateralAmount,
        address assetToken,
        uint256 collateralRatio
    ) public view returns (uint256) {
        Asset storage asset = _assetMap[assetToken];
        (int256 relativeAssetPrice, ) = IPriceFeed(asset.priceFeed)
            .getLatestPrice();
        uint8 decimal = sAsset(assetToken).decimals();
        uint256 mintAmount = (collateralAmount * (10**uint256(decimal))) /
            uint256(relativeAssetPrice) /
            collateralRatio;
        return mintAmount;
    }

    function checkRegistered(address assetToken) public view returns (bool) {
        return _assetMap[assetToken].token == assetToken;
    }

    /* TODO: implement your functions here */
    function registerAsset(
        address assetToken,
        uint256 minCollateralRatio,
        address priceFeed
    ) external override {}

    function openPosition(
        uint256 collateralAmount,
        address assetToken,
        uint256 collateralRatio
    ) external override {}

    function closePosition(uint256 positionIndex) external override {}

    function deposit(uint256 positionIndex, uint256 collateralAmount)
        external
        override
    {}

    function withdraw(uint256 positionIndex, uint256 withdrawAmount)
        external
        override
    {}

    function mint(uint256 positionIndex, uint256 mintAmount)
        external
        override
    {}

    function burn(uint256 positionIndex, uint256 burnAmount)
        external
        override
    {}
}

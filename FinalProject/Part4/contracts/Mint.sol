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
    ) external override {
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

    function openPosition(
        uint256 collateralAmount,
        address assetToken,
        uint256 collateralRatio
    ) external override {
        require(checkRegistered(assetToken), "Asset not exists");

        Asset storage asset = _assetMap[assetToken];
        uint256 minCollateralRatio = asset.minCollateralRatio;
        require(collateralRatio >= minCollateralRatio, "Not greater than MCR");

        uint256 CalAssetAmount = getMintAmount(collateralAmount, assetToken, collateralRatio);

        EUSD(collateralToken).transferFrom(msg.sender, address(this), collateralAmount);
        _currentPositionIndex = 0;
        // Position storage position = Position();
        _idxPositionMap[_currentPositionIndex] = Position({
            idx: _currentPositionIndex,
            owner: msg.sender,
            collateralAmount: collateralAmount,
            assetToken: assetToken,
            assetAmount: CalAssetAmount
        });
        _currentPositionIndex++;
        sAsset(assetToken).mint(msg.sender, CalAssetAmount);
    }

    function closePosition(uint256 positionIndex) external override {
        Position storage position = _idxPositionMap[positionIndex];
        require(position.owner == msg.sender, "Access denied");
        // burn the sAsset
        sAsset(position.assetToken).burn(msg.sender, position.assetAmount);
        // transfer money back
        EUSD(collateralToken).transfer(msg.sender, position.collateralAmount);

        position.assetAmount = 0;
        position.collateralAmount = 0;
        position.idx = 0;
        position.assetToken = address(0);
        position.owner = address(0);

    }

    function deposit(uint256 positionIndex, uint256 collateralAmount)
        external
        override
    {
        Position storage position = _idxPositionMap[positionIndex];
        require(position.owner == msg.sender, "Access denied");
        EUSD(collateralToken).transferFrom(msg.sender, address(this), collateralAmount);
        position.collateralAmount += collateralAmount;
    }

    function withdraw(uint256 positionIndex, uint256 withdrawAmount)
        external
        override
    {
        Position storage position = _idxPositionMap[positionIndex];
        require(position.collateralAmount >= withdrawAmount, "Not enough money");
        require(position.owner == msg.sender, "Access denied");

        Asset storage asset = _assetMap[position.assetToken];
        uint256 minCollateralRatio = asset.minCollateralRatio;
        require(position.collateralAmount - withdrawAmount > minCollateralRatio * position.assetAmount, "Not greater than MCR");

        EUSD(collateralToken).transfer(msg.sender, withdrawAmount);
        position.collateralAmount -= withdrawAmount;
    }

    function mint(uint256 positionIndex, uint256 mintAmount)
        external
        override
    {
        Position storage position = _idxPositionMap[positionIndex];
        require(position.owner == msg.sender, "Access denied");

        Asset storage asset = _assetMap[position.assetToken];
        uint256 minCollateralRatio = asset.minCollateralRatio;
        require(position.collateralAmount >= minCollateralRatio * (mintAmount + position.assetAmount), "Not greater than MCR");
        
        sAsset(position.assetToken).mint(msg.sender, mintAmount);
        position.assetAmount += mintAmount;
    }

    function burn(uint256 positionIndex, uint256 burnAmount)
        external
        override
    {
        Position storage position = _idxPositionMap[positionIndex];
        require(position.owner == msg.sender, "Access denied");

        position.assetAmount -= burnAmount;
        sAsset(position.assetToken).burn(msg.sender, burnAmount);
    }
}

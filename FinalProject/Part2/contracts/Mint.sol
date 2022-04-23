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

    function openPosition(
        uint256 collateralAmount,
        address assetToken,
        uint256 collateralRatio
    ) external override {
        // Make sure we obey the rules
        require(checkRegistered(assetToken), "Make sure asset is registered");
        require(
            collateralRatio >= _assetMap[assetToken].minCollateralRatio,
            "MCR is too low"
        );

        // transferring collateralAmount EUSD tokens from the message sender to the contract.
        EUSD(collateralToken).transferFrom(
            msg.sender,
            address(this),
            collateralAmount
        );

        // calculate the number of minted tokens to send to the message sender.
        uint256 newAssetAmount = getMintAmount(
            collateralAmount,
            assetToken,
            collateralRatio
        );
        address assetAddr = _assetMap[assetToken].token;
        sAsset(assetAddr).mint(msg.sender, newAssetAmount);

        _idxPositionMap[_currentPositionIndex] = Position(
            _currentPositionIndex,
            msg.sender,
            collateralAmount,
            assetToken,
            newAssetAmount
        );
        _currentPositionIndex += 1;
    }

    function closePosition(uint256 positionIndex) external override {
        // Make sure we obey the rules
        require(
            msg.sender == _idxPositionMap[positionIndex].owner,
            "Only the owner can call this function"
        );

        // Transfer EUSD tokens locked in the position to the message sender.
        EUSD(collateralToken).transfer(
            msg.sender,
            _idxPositionMap[positionIndex].collateralAmount
        );

        // Transfer the sAsset tokens from the message sender to the contract and burn these tokens.
        sAsset(_idxPositionMap[positionIndex].assetToken).burn(
            _idxPositionMap[positionIndex].owner,
            _idxPositionMap[positionIndex].assetAmount
        );

        // delete the position at the given index.
        delete _idxPositionMap[positionIndex];
    }

    function deposit(uint256 positionIndex, uint256 collateralAmount)
        external
        override
    {
        // Make sure we obey the rules
        require(
            msg.sender == _idxPositionMap[positionIndex].owner,
            "Make sure, only the owner can deposit a collateral"
        );

        // transfer deposited tokens from the sender to the contract
        EUSD(collateralToken).transferFrom(
            msg.sender,
            address(this),
            collateralAmount
        );

        //  Add collateral amount of the position at the given index
        _idxPositionMap[positionIndex].collateralAmount += collateralAmount;
    }

    function withdraw(uint256 positionIndex, uint256 withdrawAmount)
        external
        override
    {
        // Make sure we obey the rules
        require(
            msg.sender == _idxPositionMap[positionIndex].owner,
            "Make sure, only the owner can withdraw a collateral"
        );
        require(
            (_idxPositionMap[positionIndex].collateralAmount - withdrawAmount) /
                _idxPositionMap[positionIndex].assetAmount >=
                _assetMap[_idxPositionMap[positionIndex].assetToken]
                    .minCollateralRatio,
            "MCR is too low"
        );
        require(
            _idxPositionMap[positionIndex].collateralAmount >= withdrawAmount,
            "Not enough money"
        );

        // Withdraw collateral tokens from the position at the given index
        // Transfer withdrawn tokens from the contract to the sender
        EUSD(collateralToken).transfer(msg.sender, withdrawAmount);

        _idxPositionMap[positionIndex].collateralAmount -= withdrawAmount;
    }

    function mint(uint256 positionIndex, uint256 mintAmount) external override {
        // Make sure we obey the rules
        require(
            msg.sender == _idxPositionMap[positionIndex].owner,
            "Make sure, only the owner can mint a given token"
        );
        require(
            _idxPositionMap[positionIndex].collateralAmount /
                (_idxPositionMap[positionIndex].assetAmount + mintAmount) >=
                _assetMap[_idxPositionMap[positionIndex].assetToken]
                    .minCollateralRatio,
            "MCR is too low"
        );

        sAsset(_idxPositionMap[positionIndex].assetToken).mint(
            msg.sender,
            mintAmount
        );

        _idxPositionMap[positionIndex].assetAmount += mintAmount;
    }

    function burn(uint256 positionIndex, uint256 burnAmount) external override {
        // Make sure we obey the rules
        require(
            msg.sender == _idxPositionMap[positionIndex].owner,
            "Make sure, only the owner can burn a given token"
        );

        sAsset(_idxPositionMap[positionIndex].assetToken).burn(
            msg.sender,
            burnAmount
        );

        _idxPositionMap[positionIndex].assetAmount -= burnAmount;
    }
}

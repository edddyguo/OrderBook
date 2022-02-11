// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.0;

import "hardhat/console.sol";


contract ChemixTrade {
    struct OrderInfo {
        address user;
        string baseToken;
        string quoteToken;
        string side;
        uint amount;
        uint price;
    }
    address public admin;
    mapping (uint => OrderInfo) public orders;
    string private name;

    event NewOrder(address user, string baseToken, string quoteToken ,string side, uint amount, uint price);

     constructor(string memory _name) {
            console.log("Deploying and init chemix:", _name);
            name = _name;
     }

    function newOrder(uint _id,string memory _baseToken, string memory _quoteToken ,string memory _side,uint _amount, uint _price) external returns (string memory){
        orders[_id] = OrderInfo ({
           user: msg.sender,
           baseToken: _baseToken,
           quoteToken: _quoteToken,
           side: _side,
           amount: _amount,
           price: _price
        });
        emit NewOrder(msg.sender, _baseToken,  _quoteToken , _side, _amount,  _price);
        return "hello world";
    }

    function listOrders(uint _id) public view returns (OrderInfo memory _order) {
        _order = orders[_id];
    }

     function DEXName() public view returns (string memory) {
         return name;
     }
}
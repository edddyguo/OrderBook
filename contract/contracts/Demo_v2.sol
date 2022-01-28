pragma solidity ^0.8.0;

contract Demo_v2 {
    uint256 public x;
    bool private initialized;

    function initialize(uint256 _x) public {
        require(!initialized, "Contract instance has already been initialized");
        initialized = true;
        x = _x;
    }

    function increment() public {
      x = x + 1;
    }

    function increment10() public {
          x = x + 10;
    }
}
pragma solidity ^0.8.17;

import {IERC20 as IERC20_NEAR} from "openzeppelin-contracts/token/ERC20/IERC20.sol";
import "../lib/aurora-engine/etc/eth-contracts/contracts/EvmErc20.sol";
import {AuroraSdk, NEAR, PromiseCreateArgs, PromiseResultStatus, PromiseWithCallback} from "../lib/aurora-contracts-sdk/aurora-solidity-sdk/src/AuroraSdk.sol";
import "../lib/aurora-contracts-sdk/aurora-solidity-sdk/src/Borsh.sol";
import {Strings} from "@openzeppelin/contracts/utils/Strings.sol";
import "@openzeppelin/contracts/utils/Base64.sol";
import "@openzeppelin/contracts/access/AccessControl.sol";

uint64 constant INC_NEAR_GAS = 36_000_000_000_000;
uint64 constant INIT_NEAR_GAS = 100_000_000_000_000;

contract AuroraErc20FastBridge is AccessControl {
    using AuroraSdk for NEAR;
    using AuroraSdk for PromiseCreateArgs;
    using AuroraSdk for PromiseWithCallback;
    using Borsh for Borsh.Data;

    bytes32 public constant MASTER = keccak256("MASTER");
    bytes32 public constant CALLBACK_ROLE = keccak256("CALLBACK_ROLE");

    address constant WNEAR_ADDRESS = 0x4861825E75ab14553E5aF711EbbE6873d369d146;

    address creator;
    NEAR public near;

    mapping(string => EvmErc20) registered_tokens;

    event NearContractInit(string near_addres);
    event Log(string msg);
    event LogUint(uint128 msg);

    constructor() {
        creator = msg.sender;
        near = AuroraSdk.initNear(IERC20_NEAR(WNEAR_ADDRESS));

        _grantRole(CALLBACK_ROLE, AuroraSdk.nearRepresentitiveImplicitAddress(address(this)));
        _grantRole(MASTER, msg.sender);
    }

    function tokens_registration(address aurora_token_address, string memory near_token_address) public onlyRole(MASTER) {
        emit NearContractInit(string(get_near_address()));

        uint128 deposit = 12_500_000_000_000_000_000_000;
        near.wNEAR.transferFrom(msg.sender, address(this), uint256(deposit));
        bytes memory args = bytes(string.concat('{"account_id": "', string(get_near_address()), '", "registreation_only": true }'));

        registered_tokens[near_token_address] = EvmErc20(aurora_token_address);
        PromiseCreateArgs memory callInc = near.call(near_token_address, "storage_deposit", args, deposit, INC_NEAR_GAS);
        callInc.transact();
    }

    function init_token_transfer(bytes memory init_transfer_args) public {
        Borsh.Data memory borsh = Borsh.from(init_transfer_args);
        borsh.decodeU64(); //valid_till
        string memory token_address_on_near = string(borsh.decodeBytes()); //transfer token address on Near
        borsh.decodeBytes20(); //transfer token address on Ethereum
        uint128 transfer_token_amount = borsh.decodeU128();
        borsh.decodeBytes(); // fee token address on Near
        uint128 fee_token_amount = borsh.decodeU128();

        EvmErc20 token = registered_tokens[token_address_on_near];
        token.transferFrom(msg.sender, address(this), uint256(transfer_token_amount + fee_token_amount));
        token.withdrawToNear(get_near_address(), uint256(transfer_token_amount + fee_token_amount));
        near.wNEAR.transferFrom(msg.sender, address(this), uint256(1));

        string memory init_args_base64 = Base64.encode(init_transfer_args);
        bytes memory args = bytes(string.concat('{"receiver_id": "fb.olga24912_3.testnet", "amount": "', Strings.toString(transfer_token_amount + fee_token_amount), '", "msg": "', init_args_base64, '"}'));

        PromiseCreateArgs memory callTr = near.call(token_address_on_near, "ft_transfer_call", args, 1, INIT_NEAR_GAS);
        PromiseCreateArgs memory callback = near.auroraCall(address(this), abi.encodePacked(this.init_token_transfer_callback.selector), 0, INC_NEAR_GAS);

        callTr.then(callback).transact();
    }

    function init_token_transfer_callback() public onlyRole(CALLBACK_ROLE) {
        uint128 transferred_amount = 0;

        if (AuroraSdk.promiseResult(0).status == PromiseResultStatus.Successful) {
            transferred_amount = uint128(bytes16(AuroraSdk.promiseResult(0).output));
        }

        emit LogUint(transferred_amount);
    }

    function get_near_address() public view returns (bytes memory) {
        bytes memory near_address_raw = bytes(string.concat(Strings.toHexString(uint160(address(this))), ".aurora"));
        bytes memory near_address = new bytes(near_address_raw.length - 2);

        for (uint256 i = 0; i < near_address_raw.length - 2; ++i) {
            near_address[i] = near_address_raw[i + 2];
        }

        return near_address;
    }

    function destruct() public {
        // Destroys this contract and sends remaining funds back to creator
        if (msg.sender == creator)
            selfdestruct(payable(creator));
    }
}

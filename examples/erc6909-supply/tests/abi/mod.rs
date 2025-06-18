#![allow(dead_code)]
use alloy::sol;

sol!(
    #[sol(rpc)]
    contract Erc6909Supply {
        function transfer(address receiver, uint256 id, uint256 amount) external returns (bool status);
        function transferFrom(address sender, address receiver, uint256 id, uint256 amount) external returns (bool status);
        function approve(address spender, uint256 id, uint256 amount) external returns (bool status);
        function setOperator(address spender, bool approved) external returns (bool status);
        function balanceOf(address owner, uint256 id) external view returns (uint256 balance);
        function allowance(address owner, address spender, uint256 id) external view returns (uint256 balance);
        function isOperator(address owner, address spender) external returns (bool status);
        function mint(address to, uint256 id, uint256 amount) external;
        function mintBatch(address to, uint256[] memory ids, uint256[] memory amounts) external;
        function burn(address from, uint256 id, uint256 amount) external;
        function burnBatch(address from, uint256[] memory ids, uint256[] memory amounts) external;
        function totalSupply(uint256 id) external view returns (uint256);
        function supportsInterface(bytes4 interfaceId) external view returns (bool);

        error Erc6909InsufficientBalance(address sender, uint256 balance, uint256 needed, uint256 id);
        error Erc6909InsufficientPermission(address spender, uint256 id);
        error Erc6909InsufficientAllowance(address spender, uint256 allowance, uint256 needed, uint256 id);
        error ERC6909InvalidApprover(address approver);
        error ERC6909InvalidSender(address sender);
        error ERC6909InvalidSpender(address spender);
        error ERC6909InvalidReceiver(address receiver);
        error ERC6909InvalidArrayLength(uint256 ids_length, uint256 values_length);

        event Transfer(address caller, address indexed sender, address indexed receiver, uint256 indexed id, uint256 amount);
        event OperatorSet(address indexed owner, address indexed spender, bool approved);
        event Approval(address indexed owner, address indexed spender, uint256 indexed id, uint256 amount);
        #[derive(Debug, PartialEq)]
        event TransferSingle(address indexed caller, address indexed from, address indexed to, uint256 id, uint256 amount) ;
        event TransferBatch(address indexed caller, address indexed from, address indexed to, uint256[] ids, uint256[] amounts);
    }
);

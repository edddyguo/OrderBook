insert into chemix_markets (id,base_token_address,base_token_symbol,quote_token_address,quote_token_symbol,matched_address,online,up_at,down_at,created_at) values ('ETH-USDT', '0x6362fbd7c3a9e1bbdc81a61a58e036c3247d44cc07','ETH', '0x63bedfa1e1ea5891cb5f0819a7d16b7fe3aef5ddb0','USDT', '0x63cc0bfe91b31388dbd9eeafb233616bacc42cab31', true,NOW(),NOW() + '10 years',NOW());
insert into chemix_markets (id,base_token_address,base_token_symbol,quote_token_address,quote_token_symbol,matched_address,online,up_at,down_at,created_at) values ('BTC-USDT', '0x63210793010d03b04ddb61f8f219a8e7e40bcba668','BTC', '0x63bedfa1e1ea5891cb5f0819a7d16b7fe3aef5ddb0','USDT', '0x638205ec560b3082ab2956ace07280fe2c251210b8',true,NOW(),NOW() + '10 years',NOW());




insert into chemix_tokens (address, symbol, name,decimals,bsc_address,bsc_assetid) values ('0x63210793010d03b04ddb61f8f219a8e7e40bcba668', 'BTC','BTCC', 8,'0x63ed080e7f11494e7563fff04668dfddc1555398de','000000000000000200000001');
insert into chemix_tokens (address, symbol, name,decimals,bsc_address,bsc_assetid) values ('0x63bedfa1e1ea5891cb5f0819a7d16b7fe3aef5ddb0', 'USDT','USDTS', 8,'0x63ed080e7f11494e7563fff04668dfddc1555398de','000000000000000200000003');
insert into chemix_tokens (address, symbol, name,decimals,bsc_address,bsc_assetid) values ('0x6362fbd7c3a9e1bbdc81a61a58e036c3247d44cc07', 'ETH','ETHP', 8,'0x63ed080e7f11494e7563fff04668dfddc1555398de','000000000000000200000002');

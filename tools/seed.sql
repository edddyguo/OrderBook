--insert into chemix_markets (id,base_token_address,base_token_symbol,base_front_decimal,base_contract_decimal,quote_token_address,quote_token_symbol,quote_front_decimal,quote_contract_decimal,online,up_at,down_at,created_at) values ('BTC-USDT', '0x3e1A99f4Ebdec4F6Da224D54a4a25b7B1445e1ea','BTC', 8,18,'0x707c73B9425276c0c0adcdd0d1178bB541792049','USDT', 8,15, true,NOW(),NOW() + '10 years',NOW());
--local
insert into chemix_markets (id,base_token_address,base_token_symbol,base_front_decimal,base_contract_decimal,quote_token_address,quote_token_symbol,quote_front_decimal,quote_contract_decimal,online,up_at,down_at,created_at) values ('BTC-USDT', '0x1785f0481CA0a369061802548444b3162B19070b','BTC', 8,18,'0x937Eb6B6d2803e627B06270B732866B9B0E5E71d','USDT', 8,15, true,NOW(),NOW() + '10 years',NOW());
--insert into chemix_markets (id,base_token_address,base_token_symbol,base_front_decimal,base_contract_decimal,quote_token_address,quote_token_symbol,base_contract_decimal,quote_contract_decimal,up_at,down_at,created_at) values ('AAA-CCC', '0x18D5034280703EA96e36a50f6178E43565eaDc67','AAA', 11,8,'0x7E62F80cA349DB398983E2Ee1434425f5B888f42','CCC',true,NOW(),NOW() + '10 years',NOW());

--insert into chemix_tokens (address, symbol, name,decimals,front_decimal) values ('0x18D5034280703EA96e36a50f6178E43565eaDc67', 'AAA','AAAA', 11,8);
--insert into chemix_tokens (address, symbol, name,decimals,front_decimal) values ('0x7E62F80cA349DB398983E2Ee1434425f5B888f42', 'BBB','BBBB', 22,8);
--insert into chemix_tokens (address, symbol, name,decimals,front_decimal) values ('0x7E62F80cA349DB398983E2Ee1434425f5B888f42', 'CCC','CCCC', 18,8);
--deployTokenA:   0x12B4e1E58D2EEc9B984A18D7275359E269726Dc2
--deployTokenB:   0x1B1D8299C787046dE1Be0CCb80aBfeb7Bf126809
--
--deployTokenA:   0x92177d3e7be191Eb7537299ae1f266de5d2fE939
--deployTokenB:   0x0eDf2C0379Dba54dDf980cc58666F3698C76f640

--deployTokenA:   0x93E139a29b5bfe61Ae34B1D8E526C4Db1A8291ef
--deployTokenB:   0x0ffB2710A3e25370C987fA52e906459d4c03e105

insert into chemix_markets (id,base_token_address,base_token_symbol,base_front_decimal,base_contract_decimal,quote_token_address,quote_token_symbol,quote_front_decimal,quote_contract_decimal,online,up_at,down_at,created_at) values ('CCC-CHE', '0x75cee65DCf0EA58801779FF716156eEB0bebb2C8','CCC', 8,18,'0x0702f6Ce4d63c0F81458F20b566eaC652EA669BF','CHE', 8,15, true,NOW(),NOW() + '10 years',NOW());
deployTokenA:   0x1785f0481CA0a369061802548444b3162B19070b
deployTokenB:   0x937Eb6B6d2803e627B06270B732866B9B0E5E71d
deployTokenC:   0x75cee65DCf0EA58801779FF716156eEB0bebb2C8
deployTokenCHE:   0x0702f6Ce4d63c0F81458F20b566eaC652EA669BF
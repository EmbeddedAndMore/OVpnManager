-- Your SQL goes here
CREATE TABLE `hosts`(
	`id` INTEGER NOT NULL PRIMARY KEY,
	`name` TEXT NOT NULL,
	`ip_address` TEXT NOT NULL,
	`username` TEXT NOT NULL,
	`password` TEXT NOT NULL
);

CREATE TABLE `vpns`(
	`id` INTEGER NOT NULL PRIMARY KEY,
	`host_id` INTEGER NOT NULL,
	`name` TEXT NOT NULL,
	`port` INT2 NOT NULL,
	`subnet` TEXT NOT NULL
);

CREATE TABLE `users`(
	`id` INTEGER NOT NULL PRIMARY KEY,
	`vpn_id` INTEGER NOT NULL,
	`username` TEXT NOT NULL,
	`is_plant` BOOL NOT NULL,
	`ca_key` TEXT NOT NULL,
	`password` TEXT NOT NULL
);


-- ---------------------------------------------
-- DB Schema for the API access token management
-- ---------------------------------------------

DROP TABLE IF EXISTS `ApiUser`;
CREATE TABLE `ApiUser` (
    `id` INT auto_increment NOT NULL,
    `name` varchar(40) NULL,
    `email` varchar(80) NOT NULL,
    `validated` BOOL DEFAULT false NULL,
    `enabled` BOOL DEFAULT false NULL,
    CONSTRAINT `ApiUser_PK` PRIMARY KEY (`id`)
);

DROP TABLE IF EXISTS `ApiToken`;
CREATE TABLE `ApiToken` (
    `created` TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    `api_token` varchar(25) NOT NULL,
    `valid_until` DATE NOT NULL,
    `client_id` INT NOT NULL,
    CONSTRAINT `ApiToken_PK` PRIMARY KEY (`api_token`),
    CONSTRAINT `ApiToken_ApiUser_FK` FOREIGN KEY (`client_id`) REFERENCES `ApiUser` (`id`) ON DELETE CASCADE
);

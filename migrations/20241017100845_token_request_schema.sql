-- ---------------------------------------------
-- DB Schema for the API access token management
-- ---------------------------------------------

DROP TABLE IF EXISTS `ApiUser`;
CREATE TABLE `ApiUser` (
    `id` VARCHAR(36) NOT NULL,
    `name` varchar(40) NULL,
    `email` varchar(80) NOT NULL,
    `validated` BOOL DEFAULT false NULL,
    `enabled` BOOL DEFAULT false NULL,
    `explanation` TEXT NOT NULL,
    CONSTRAINT `ApiUser_PK` PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci;

DROP TABLE IF EXISTS `ApiToken`;
CREATE TABLE `ApiToken` (
    `created` TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    `api_token` varchar(100) NOT NULL,
    `valid_until` TIMESTAMP NOT NULL,
    `client_id` VARCHAR(36) NOT NULL,
    CONSTRAINT `ApiToken_PK` PRIMARY KEY (`api_token`),
    CONSTRAINT `ApiToken_ApiUser_FK` FOREIGN KEY (`client_id`) REFERENCES `ApiUser` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci;

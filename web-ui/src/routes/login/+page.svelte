<div class="container">
	<div class="login-box">
		<form method="post" action="/api/login">
			<h2>Login</h2>
			<label>Username: <input name="username"/></label>
			<label>Password: <input type="password" name="password"/></label>
			<input type="submit" value="Login"/>
		</form>
	</div>
</div>

<style lang="scss">
	.container {
		margin: 0;
		display: flex;
		align-items: center;
		justify-content: center;
		width: 100vw;
		height: 100vh;
	}
	
	h2 {
		margin-top: 0;
		margin-bottom: 0;
	}
	
	.login-box {
		background-color: #36363B;
		padding: 20px;
		
		form {
			display: flex;
			flex-direction: column;
			gap: 8px;
			
			input[type=submit] {
				align-self: start;
			}
		}
	}
</style>
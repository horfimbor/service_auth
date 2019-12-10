class Login extends HTMLElement {
    constructor() {
        super();

        this.attachShadow({ mode: 'open' });
        this.shadowRoot.innerHTML = `
            <form class="login" method="post">
                <input minlength="5" placeholder="passphrase" type="password" name="passphrase" class="passphrase">
                <input type="submit" value="login"/>
            </form>
        `
    }

    connectedCallback() {
        this.shadowRoot.querySelector("form.login").addEventListener('submit', this._login.bind(this));
    }

    disconnectedCallback() {
        this.shadowRoot.querySelector("form.login").removeEventListener('submit', this._login);
    }

    _login(e) {
        e.preventDefault();

        console.debug(this.shadowRoot.querySelector(".passphrase").value)

        fetch("http://localhost:8000/login", {
                method : "POST",
                headers: {
                   cache : "no-cache"
                },
                body : JSON.stringify({
                    passphrase: this.shadowRoot.querySelector(".passphrase").value
                }),
            })
            .then(res => res.text()) // parse response as JSON with res.json
            .then(response => {

                if(response === "data_required"){
                    import('http://localhost:8000/js/signup.js').then(module => {
                      console.log('signup loaded')
                    });
                    this.shadowRoot.innerHTML = `<hf-auth-signup> </hf-auth-signup>  `
                }else{
                    console.log({service:"auth", status:"ok", resp:response})
                }


            })
            .catch(err => {
                console.log({service:"auth", status:"KO", error:err})
                alert("sorry, cannot auth")
            });
    }
}
customElements.define('hf-auth-login', Login);